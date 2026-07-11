use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU32};

use anyhow::{Context, Result};

use crate::hooks::ffi::QuestState;
use crate::process::Process;

pub static QUEST_STATE_PTR: AtomicPtr<QuestState> = AtomicPtr::new(ptr::null_mut());
pub static PLAYER_DATA_OFFSET: AtomicU32 = AtomicU32::new(0);
pub static WEAPON_OFFSET: AtomicU32 = AtomicU32::new(0);
pub static OVERMASTERY_OFFSET: AtomicU32 = AtomicU32::new(0);
pub static SIGIL_OFFSET: AtomicU32 = AtomicU32::new(0);
pub static SBA_OFFSET: AtomicU32 = AtomicU32::new(0);

/// Game 2.0 restructured the property lookup this used to resolve via a static byte
/// pattern (it now goes through a runtime hashtable dispatch instead of a fixed
/// compile-time offset). Verified live on 2026-07-11 against a known character's
/// level/HP/attack/power values via a bounded memory scan from the damage hook -
/// see feedback_re_methodology memory for the technique. Re-verify if this ever
/// silently produces wrong player stats after a future game patch.
const PLAYER_DATA_OFFSET_GAME_2: u32 = 0x15030;

/// Unlike player_data_offset, this is a *direct* offset from the entity to a slot
/// that holds a pointer to a separately-allocated SigilList (confirmed live:
/// entity_ptr+this, read as a pointer, dereferenced, gives sigil_id/sigil_level
/// matching the actually-equipped sigil). See feedback_re_methodology memory - this
/// needed an extra level of indirection beyond what worked for player_data_offset.
const SIGIL_OFFSET_GAME_2: u32 = 0x1AE90;

pub fn setup_globals(process: &Process) -> Result<()> {
    let player_data_offset = PLAYER_DATA_OFFSET_GAME_2;

    #[cfg(feature = "console")]
    println!("player_data_offset: {:x}", player_data_offset);

    PLAYER_DATA_OFFSET.store(player_data_offset, std::sync::atomic::Ordering::Relaxed);

    #[cfg(feature = "console")]
    println!("sigil_offset: {:x}", SIGIL_OFFSET_GAME_2);

    SIGIL_OFFSET.store(SIGIL_OFFSET_GAME_2, std::sync::atomic::Ordering::Relaxed);

    let weapon_offset = process
        .search_slice::<u8>("48 ? ? ' ? 48 ? ? ? 48 ? ? e8 ? ? ? ? 31 ?")
        .context("Could not find weapon offset")?;

    #[cfg(feature = "console")]
    println!("weapon_offset: {:x}", weapon_offset);

    WEAPON_OFFSET.store(
        player_data_offset + weapon_offset as u32,
        std::sync::atomic::Ordering::Relaxed,
    );

    let overmastery_offset = process
        .search_slice::<u32>("49 8D 8C 24 ' ? ? ? ? 48 8D 93 ? ? ? ? E8 ? ? ? ?")
        .context("Could not find overmastery offset")?;

    #[cfg(feature = "console")]
    println!("overmastery_offset: {:x}", overmastery_offset);

    OVERMASTERY_OFFSET.store(
        player_data_offset + overmastery_offset,
        std::sync::atomic::Ordering::Relaxed,
    );

    let sba_offset = process
        .search_slice::<u32>("7E ? C5 FA 59 81 ? ? ? ? 48 81 C1 ' ? ? ? ? C5 F8 54 0D ? ? ? ?")
        .context("Could not find sba offset")?;

    #[cfg(feature = "console")]
    println!("sba_offset: {:x}", sba_offset);

    SBA_OFFSET.store(sba_offset, std::sync::atomic::Ordering::Relaxed);

    Ok(())
}
