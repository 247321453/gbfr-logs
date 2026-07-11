use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::NonNull;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use protocol::{ActionType, Actor, DamageEvent, Message};
use retour::static_detour;

use crate::{
    event,
    hooks::{ffi::DamageInstance, ffi::PlayerStats, globals::PLAYER_DATA_OFFSET},
    process::Process,
};

use super::{actor_idx, actor_type_id, get_source_parent};

/// Piggybacks on the damage hook to opportunistically send player stats, since the
/// dedicated "on load player" hook's signature is stale for game 2.0 (see
/// project_game_2_compatibility_fix memory) and re-deriving it live proved too
/// fragile (address instability across quest loads, crashes when watching
/// character-switch memory operations). This sidesteps needing that event at all:
/// every damage hit already gives us a live pointer to the attacker's entity.
const PLAYER_STATS_RESEND_INTERVAL: Duration = Duration::from_secs(5);

fn maybe_send_player_stats(tx: &event::Tx, actor_index: u32, character_type: u32, entity_ptr: *const usize) {
    let player_offset = PLAYER_DATA_OFFSET.load(Ordering::Relaxed);

    if player_offset == 0 {
        return;
    }

    static LAST_SENT: OnceLock<Mutex<HashMap<u32, Instant>>> = OnceLock::new();
    let mut last_sent = LAST_SENT.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    let now = Instant::now();

    if let Some(sent_at) = last_sent.get(&actor_index) {
        if now.duration_since(*sent_at) < PLAYER_STATS_RESEND_INTERVAL {
            return;
        }
    }

    let raw_player_stats =
        std::ptr::NonNull::new(unsafe { entity_ptr.byte_add(player_offset as usize) } as *mut PlayerStats);

    let Some(raw_player_stats) = raw_player_stats else {
        return;
    };

    let stats = unsafe { raw_player_stats.as_ref() };

    // Enemies/pets don't have a real PlayerStats struct at this offset; a sane-looking
    // level/health/power triple simultaneously is a strong signal this really is a
    // player (matches the same fields used to originally verify this offset live).
    let looks_like_player = (1..=999).contains(&stats.level)
        && (1..10_000_000).contains(&stats.total_health)
        && (1..1_000_000).contains(&stats.total_power);

    if !looks_like_player {
        return;
    }

    last_sent.insert(actor_index, now);

    let payload = Message::PlayerLoadEvent(protocol::PlayerLoadEvent {
        sigils: Vec::new(),
        character_name: CString::new("").unwrap(),
        display_name: CString::new("").unwrap(),
        actor_index,
        is_online: false,
        // Unknown without sigil_offset; consumers key off actor_index instead.
        party_index: 0xFF,
        player_stats: protocol::PlayerStats {
            level: stats.level,
            total_hp: stats.total_health,
            total_attack: stats.total_attack,
            stun_power: stats.stun_power,
            critical_rate: stats.critical_rate,
            total_power: stats.total_power,
        },
        character_type,
        weapon_info: None,
        overmastery_info: None,
    });

    let _ = tx.send(payload);
}

type ProcessDamageEventFunc =
    unsafe extern "system" fn(*const usize, *const usize, *const usize, u8) -> usize;

type ProcessDotEventFunc = unsafe extern "system" fn(*const usize, *const usize) -> usize;

static_detour! {
    static ProcessDamageEvent: unsafe extern "system" fn(*const usize, *const usize, *const usize, u8) -> usize;
    static ProcessDotEvent: unsafe extern "system" fn(*const usize, *const usize) -> usize;
}

#[derive(Clone)]
pub struct OnProcessDamageHook {
    tx: event::Tx,
}

const PROCESS_DAMAGE_EVENT_SIG: &str = "e8 $ { ' } 66 83 bc 24 ? ? ? ? ?";

impl OnProcessDamageHook {
    pub fn new(tx: event::Tx) -> Self {
        OnProcessDamageHook { tx }
    }

    pub fn setup(&self, process: &Process) -> Result<()> {
        let cloned_self = self.clone();

        if let Ok(process_dmg_evt) = process.search_address(PROCESS_DAMAGE_EVENT_SIG) {
            #[cfg(feature = "console")]
            println!("Found process dmg event");

            unsafe {
                let func: ProcessDamageEventFunc = std::mem::transmute(process_dmg_evt);

                ProcessDamageEvent
                    .initialize(func, move |a1, a2, a3, a4| cloned_self.run(a1, a2, a3, a4))?;

                ProcessDamageEvent.enable()?;
            }
        } else {
            return Err(anyhow!("Could not find process_dmg_evt"));
        }

        Ok(())
    }

    fn run(&self, a1: *const usize, a2: *const usize, a3: *const usize, a4: u8) -> usize {
        // Target is the instance of the actor being damaged.
        // For example: Instance of the Em2700 class.
        let target_specified_instance_ptr: usize = unsafe { *(*a1.byte_add(0x08) as *const usize) };

        let original_value = unsafe { ProcessDamageEvent.call(a1, a2, a3, a4) };

        // This points to the first Entity instance in the 'a2' entity list.
        let source_entity_ptr = unsafe { (a2.byte_add(0x18) as *const *const usize).read() };

        // @TODO(false): For some reason, online + Ferry's Umlauf skill pet can return a null pointer here.
        // Possible data race with online?
        if source_entity_ptr.is_null() {
            return original_value;
        }

        // entity->m_pSpecifiedInstance, offset 0x70 from entity pointer.
        // Returns the specific class instance of the source entity. (e.g. Instance of Pl1200 / Pl0700Ghost)
        let source_specified_instance_ptr: usize = unsafe { *(source_entity_ptr.byte_add(0x70)) };

        let damage_instance = unsafe { NonNull::new(a2 as *mut DamageInstance).unwrap().as_ref() };
        let damage: i32 = damage_instance.damage;

        if original_value == 0 || damage <= 0 {
            return original_value;
        }

        let flags: u64 = damage_instance.flags;

        let action_type: ActionType = if ((1 << 7 | 1 << 50) & flags) != 0 {
            ActionType::LinkAttack
        } else if ((1 << 13 | 1 << 14) & flags) != 0 {
            ActionType::SBA
        } else if ((1 << 15) & flags) != 0 {
            ActionType::SupplementaryDamage(damage_instance.action_id)
        } else {
            ActionType::Normal(damage_instance.action_id)
        };

        // Get the source actor's type ID.
        let source_type_id = actor_type_id(source_specified_instance_ptr as *const usize);
        let source_idx = actor_idx(source_specified_instance_ptr as *const usize);

        maybe_send_player_stats(
            &self.tx,
            source_idx,
            source_type_id,
            source_specified_instance_ptr as *const usize,
        );

        // Parent layouts are character-specific and changed in the 2.0 update. Keep the
        // source attributed to the concrete actor until those optional offsets are verified.
        let (source_parent_type_id, source_parent_idx) = (source_type_id, source_idx);

        let target_type_id: u32 = actor_type_id(target_specified_instance_ptr as *const usize);
        let target_idx = actor_idx(target_specified_instance_ptr as *const usize);

        let event = Message::DamageEvent(DamageEvent {
            source: Actor {
                index: source_idx,
                actor_type: source_type_id,
                parent_index: source_parent_idx,
                parent_actor_type: source_parent_type_id,
            },
            target: Actor {
                index: target_idx,
                actor_type: target_type_id,
                parent_index: target_idx,
                parent_actor_type: target_type_id,
            },
            damage,
            flags,
            action_id: action_type,
            attack_rate: None,
            damage_cap: Some(damage_instance.damage_cap),
            stun_value: None,
        });

        let _ = self.tx.send(event);

        original_value
    }
}

#[derive(Clone)]
pub struct OnProcessDotHook {
    tx: event::Tx,
}

impl OnProcessDotHook {
    pub fn new(tx: event::Tx) -> Self {
        OnProcessDotHook { tx }
    }

    pub fn setup(&self, process: &Process) -> Result<()> {
        let cloned_self = self.clone();

        if let Ok(process_dot_evt) =
            process.search_address("44 89 74 24 ? 48 ? ? ? ? 48 ? ? e8 $ { ' } 4c")
        {
            #[cfg(feature = "console")]
            println!("Found process dot event");

            unsafe {
                let func: ProcessDotEventFunc = std::mem::transmute(process_dot_evt);
                ProcessDotEvent.initialize(func, move |a1, a2| cloned_self.run(a1, a2))?;
                ProcessDotEvent.enable()?;
            }
        } else {
            return Err(anyhow!("Could not find process_dot_evt"));
        }

        Ok(())
    }

    // A1: DoT Instance (StatusPl2300ParalysisArrow)
    // *A1+0x00 -> StatusAilmentPoison : StatusBase
    // A1+0x18->targetEntityInfo : CEntityInfo (Target entity of the DoT, what is being damaged)
    // A1+0x30->sourceEntityInfo : CEntityInfo (Source entity of the DoT, who applied it)
    // A1+0x50->duration : float (How much time is left for the DoT)
    fn run(&self, dot_instance: *const usize, a2: *const usize) -> usize {
        let original_value = unsafe { ProcessDotEvent.call(dot_instance, a2) };

        // @TODO(false): There's a better way to check null pointers with Option type, but I'm too dumb to figure it out right now.
        let target_info = unsafe { dot_instance.byte_add(0x18).read() } as *const usize;
        let source_info = unsafe { dot_instance.byte_add(0x30).read() } as *const usize;

        if target_info.is_null() || source_info.is_null() {
            return original_value;
        }

        let target = unsafe { target_info.byte_add(0x70).read() } as *const usize;
        let source = unsafe { source_info.byte_add(0x70).read() } as *const usize;

        if target.is_null() || source.is_null() {
            return original_value;
        }

        let dmg = unsafe { (a2 as *const i32).read() };

        let source_idx = actor_idx(source);
        let source_type_id = actor_type_id(source);

        let target_idx = actor_idx(target);
        let target_type_id = actor_type_id(target);

        let (source_parent_type_id, source_parent_idx) =
            get_source_parent(source_type_id, source).unwrap_or((source_type_id, source_idx));

        let event = Message::DamageEvent(DamageEvent {
            source: Actor {
                index: source_idx,
                actor_type: source_type_id,
                parent_index: source_parent_idx,
                parent_actor_type: source_parent_type_id,
            },
            target: Actor {
                index: target_idx,
                actor_type: target_type_id,
                parent_index: target_idx,
                parent_actor_type: target_type_id,
            },
            damage: dmg,
            flags: 0,
            action_id: ActionType::DamageOverTime(0),
            attack_rate: None,
            stun_value: None,
            damage_cap: None,
        });

        let _ = self.tx.send(event);

        original_value
    }
}
