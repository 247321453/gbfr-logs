import { CaretDown, CaretUp } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

import { ComputedPlayerState, PlayerData } from "@/types";
import { darkenColorToRgba, translatedPlayerName } from "@/utils";

import { usePlayerCard } from "./usePlayerCard";

export const PlayerCard = ({
  player,
  partyData,
  isOpen,
  onToggle,
}: {
  player: ComputedPlayerState;
  partyData: Array<PlayerData | null>;
  isOpen: boolean;
  onToggle: () => void;
}) => {
  const { t } = useTranslation();
  const { color, partySlotIndex, showDisplayNames, primaryColumn, primaryValue, secondaryColumn, secondaryValue } =
    usePlayerCard(player, partyData);

  // Darker + translucent version of the player's own bar color, so the card background
  // reads as related to that player without blending into the brighter bar itself.
  const cardBackground = darkenColorToRgba(color, 0.4, 0.3);

  return (
    <div
      className={`player-card ${isOpen ? "player-card-open" : ""}`}
      style={{ backgroundColor: cardBackground }}
      onClick={onToggle}
    >
      <div className="player-card-name" title={translatedPlayerName(partySlotIndex, partyData[partySlotIndex], player, showDisplayNames)}>
        {translatedPlayerName(partySlotIndex, partyData[partySlotIndex], player, showDisplayNames)}
      </div>

      <div className="player-card-stats">
        {primaryValue && (
          <span className="player-card-stat-primary">
            {primaryValue.value}
            {primaryValue.unit && <span className="unit font-sm">{primaryValue.unit}</span>}
          </span>
        )}
        {secondaryColumn && secondaryValue && (
          <span className="player-card-stat-secondary">
            {secondaryValue.value}
            {secondaryValue.unit && <span className="unit font-sm">{secondaryValue.unit}</span>}
          </span>
        )}
        <span className="player-card-caret">{isOpen ? <CaretUp size={12} /> : <CaretDown size={12} />}</span>
      </div>

      <div className="player-card-bar-track">
        <div className="player-card-bar" style={{ backgroundColor: color, width: `${player.percentage}%` }} />
      </div>

      {primaryColumn && <div className="player-card-label">{t(`ui.meter-columns.${primaryColumn}`)}</div>}
    </div>
  );
};
