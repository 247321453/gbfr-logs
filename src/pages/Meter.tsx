import { Toaster } from "react-hot-toast";
import { useShallow } from "zustand/react/shallow";

import { PlayerStrip } from "@/components/live/PlayerStrip";
import { Table } from "@/components/Table";
import { Titlebar } from "@/components/Titlebar";
import { useMeterSettingsStore } from "@/stores/useMeterSettingsStore";
import { MeterTheme } from "@/types";
import "@/i18n";

import "./Meter.css";
import useMeter from "./useMeter";

export const Meter = () => {
  const {
    encounterState,
    partyData,
    lastPartyData,
    elapsedTime,
    sortType,
    setSortType,
    sortDirection,
    setSortDirection,
    transparency,
  } = useMeter();
  const { meterTheme } = useMeterSettingsStore(useShallow((state) => ({ meterTheme: state.meter_theme })));

  const isHorizontalOverlay = meterTheme === MeterTheme.HorizontalOverlay;
  const activePartyData = encounterState.status === "Stopped" ? lastPartyData : partyData;

  return (
    <div className="app">
      {isHorizontalOverlay ? (
        <div data-tauri-drag-region className="drag-handle" />
      ) : (
        <Titlebar
          encounterState={encounterState}
          partyData={activePartyData}
          elapsedTime={elapsedTime}
          sortType={sortType}
          sortDirection={sortDirection}
        />
      )}
      <div
        className={`app-content ${isHorizontalOverlay ? "app-content-no-titlebar" : ""}`}
        style={{ background: `rgba(22, 22, 22, ${transparency})` }}
      >
        {isHorizontalOverlay ? (
          <PlayerStrip
            encounterState={encounterState}
            partyData={activePartyData}
            sortType={sortType}
            sortDirection={sortDirection}
          />
        ) : (
          <Table
            live
            encounterState={encounterState}
            partyData={activePartyData}
            sortType={sortType}
            setSortType={setSortType}
            sortDirection={sortDirection}
            setSortDirection={setSortDirection}
          />
        )}
      </div>
      <Toaster
        position="bottom-center"
        toastOptions={{
          style: {
            borderRadius: "10px",
            backgroundColor: "#252525",
            color: "#fff",
            fontSize: "14px",
          },
        }}
      />
    </div>
  );
};
