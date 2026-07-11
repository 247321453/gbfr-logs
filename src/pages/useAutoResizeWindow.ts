import { LogicalPosition, LogicalSize, appWindow, currentMonitor } from "@tauri-apps/api/window";
import { RefObject, useEffect } from "react";

// Roughly matches a card's natural width in Meter.css (flex-basis 120px + padding), so
// the window fits N cards without them stretching or squeezing awkwardly.
const CARD_WIDTH = 150;
const MIN_WINDOW_WIDTH = 260;
const MIN_WINDOW_HEIGHT = 50;
// Matches Meter.css's .drag-handle height, which sits above the measured content.
const DRAG_HANDLE_HEIGHT = 6;

/// Resizes and re-centers the overlay window at the top of its current monitor to hug its
/// actual content: width from the number of player cards, height from the real rendered
/// height of contentRef's element (so expanding a card's skill breakdown grows the window
/// and collapsing it shrinks back down).
export const useAutoResizeWindow = (
  contentRef: RefObject<HTMLElement>,
  playerCount: number,
  enabled: boolean
) => {
  useEffect(() => {
    if (!enabled || playerCount <= 0) return;

    const node = contentRef.current;
    if (!node) return;

    let cancelled = false;
    let frame: number | null = null;

    const resize = async () => {
      const monitor = await currentMonitor();
      if (!monitor || cancelled) return;

      const scaleFactor = monitor.scaleFactor;
      const monitorWidth = monitor.size.width / scaleFactor;
      const monitorX = monitor.position.x / scaleFactor;
      const monitorY = monitor.position.y / scaleFactor;

      const targetWidth = Math.min(Math.max(playerCount * CARD_WIDTH, MIN_WINDOW_WIDTH), monitorWidth);
      const targetHeight = Math.max(node.scrollHeight + DRAG_HANDLE_HEIGHT, MIN_WINDOW_HEIGHT);

      await appWindow.setSize(new LogicalSize(targetWidth, targetHeight));

      const targetX = monitorX + (monitorWidth - targetWidth) / 2;
      await appWindow.setPosition(new LogicalPosition(targetX, monitorY));
    };

    const observer = new ResizeObserver(() => {
      if (frame !== null) cancelAnimationFrame(frame);
      frame = requestAnimationFrame(resize);
    });

    observer.observe(node);
    resize();

    return () => {
      cancelled = true;
      observer.disconnect();
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [contentRef, playerCount, enabled]);
};
