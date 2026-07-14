import { useRef, useState } from "react";
import { useShallow } from "zustand/react/shallow";

import { SkillBreakdown } from "@/components/SkillBreakdown";
import { useMeterSettingsStore } from "@/stores/useMeterSettingsStore";
import { ComputedPlayerState, EncounterState, PlayerData, SortDirection, SortType } from "@/types";
import { formatInPartyOrder, getPlayerColor, sortPlayers } from "@/utils";

import { PlayerCard } from "./PlayerCard";
import { useAutoResizeWindow } from "@/pages/useAutoResizeWindow";

export const PlayerStrip = ({
  encounterState,
  partyData,
  sortType,
  sortDirection,
}: {
  encounterState: EncounterState;
  partyData: Array<PlayerData | null>;
  sortType: SortType;
  sortDirection: SortDirection;
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [expandedPlayerIndex, setExpandedPlayerIndex] = useState<number | null>(null);
  const { streamerMode, autoResizeWindow, color_1, color_2, color_3, color_4 } = useMeterSettingsStore(
    useShallow((state) => ({
      streamerMode: state.streamer_mode,
      autoResizeWindow: state.auto_resize_window,
      color_1: state.color_1,
      color_2: state.color_2,
      color_3: state.color_3,
      color_4: state.color_4,
    }))
  );

  const partyOrderPlayers = formatInPartyOrder(encounterState.party);
  let players: Array<ComputedPlayerState> = partyOrderPlayers.map((playerData) => ({
    ...playerData,
    percentage: (playerData.totalDamage / encounterState.totalDamage) * 100,
  }));

  sortPlayers(players, sortType, sortDirection);

  players = players.filter((player) => {
    const partySlotIndex = partyData.findIndex((partyMember) => partyMember?.actorIndex === player.index);

    // If streamer mode is ON, then only show the first party slot (the streamer's character).
    return streamerMode ? partySlotIndex === 0 : true;
  });

  const expandedPlayer = players.find((player) => player.index === expandedPlayerIndex) ?? null;
  const expandedColor = expandedPlayer
    ? getPlayerColor([color_1, color_2, color_3, color_4], partyData, expandedPlayer)
    : null;

  useAutoResizeWindow(containerRef, players.length, autoResizeWindow);

  return (
    <div data-tauri-drag-region className="player-strip-container" ref={containerRef}>
      <div className="player-strip">
        {players.map((player) => (
          <PlayerCard
            key={player.index}
            player={player}
            partyData={partyData}
            isOpen={expandedPlayerIndex === player.index}
            onToggle={() => setExpandedPlayerIndex((prev) => (prev === player.index ? null : player.index))}
          />
        ))}
      </div>

      {expandedPlayer && expandedColor && (
        <table className="w-full">
          <tbody>
            <SkillBreakdown player={expandedPlayer} color={expandedColor} />
          </tbody>
        </table>
      )}
    </div>
  );
};
