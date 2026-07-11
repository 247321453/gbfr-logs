import { useState } from "react";
import { useShallow } from "zustand/react/shallow";

import { useMeterSettingsStore } from "@/stores/useMeterSettingsStore";
import { ComputedPlayerState, MeterColumns, PlayerData } from "@/types";
import { getPlayerColor, matchColumnTypeToValue as matchColumnTypeToValueUtil } from "@/utils";

export type { ColumnValue } from "@/utils";

export const usePlayerRow = (live: boolean, player: ComputedPlayerState, partyData: Array<PlayerData | null>) => {
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

  const [isOpen, setIsOpen] = useState(false);

  const color = getPlayerColor([color_1, color_2, color_3, color_4], partyData, player);
  const partySlotIndex = partyData.findIndex((partyMember) => partyMember?.actorIndex === player.index);

  // Function for matching the column type to the value to display in the table.
  const matchColumnTypeToValue = (showFullValues: boolean, column: MeterColumns) =>
    matchColumnTypeToValueUtil(player, showFullValues, column);

  // If the meter is in live mode, only show the overlay columns that are enabled, otherwise show all columns.
  const columns = live
    ? overlay_columns
    : [
        MeterColumns.TotalDamage,
        MeterColumns.DPS,
        MeterColumns.TotalStunValue,
        MeterColumns.StunPerSecond,
        MeterColumns.DamagePercentage,
      ];

  return {
    columns,
    isOpen,
    setIsOpen,
    color,
    matchColumnTypeToValue,
    partySlotIndex,
    showFullValues: show_full_values,
    showDisplayNames: show_display_names,
  };
};
