import { useShallow } from "zustand/react/shallow";

import { useMeterSettingsStore } from "@/stores/useMeterSettingsStore";
import { ComputedPlayerState, PlayerData } from "@/types";
import { getPlayerColor, matchColumnTypeToValue } from "@/utils";

export const usePlayerCard = (player: ComputedPlayerState, partyData: Array<PlayerData | null>) => {
  const { color_1, color_2, color_3, color_4, show_display_names, show_full_values, overlay_columns } =
    useMeterSettingsStore(
      useShallow((state) => ({
        color_1: state.color_1,
        color_2: state.color_2,
        color_3: state.color_3,
        color_4: state.color_4,
        show_display_names: state.show_display_names,
        show_full_values: state.show_full_values,
        overlay_columns: state.overlay_columns,
      }))
    );

  const color = getPlayerColor([color_1, color_2, color_3, color_4], partyData, player);
  const partySlotIndex = partyData.findIndex((partyMember) => partyMember?.actorIndex === player.index);

  // The compact card only has room for a primary + secondary stat, taken from the
  // user's existing overlay column order (already configurable/reorderable in Settings).
  const [primaryColumn, secondaryColumn] = overlay_columns;
  const primaryValue = primaryColumn ? matchColumnTypeToValue(player, show_full_values, primaryColumn) : null;
  const secondaryValue = secondaryColumn ? matchColumnTypeToValue(player, show_full_values, secondaryColumn) : null;

  return {
    color,
    partySlotIndex,
    showDisplayNames: show_display_names,
    primaryColumn,
    primaryValue,
    secondaryColumn,
    secondaryValue,
  };
};
