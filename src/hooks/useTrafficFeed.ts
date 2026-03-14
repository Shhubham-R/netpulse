import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTrafficStore, ConnectionEvent, SpeedStats } from "../store/trafficStore";

export function useTrafficFeed() {
  const addTrafficEvents = useTrafficStore((s) => s.addTrafficEvents);
  const updateSpeedStats = useTrafficStore((s) => s.updateSpeedStats);
  const setCaptureRunning = useTrafficStore((s) => s.setCaptureRunning);
  const isListening = useRef(false);

  useEffect(() => {
    if (isListening.current) return;
    isListening.current = true;

    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const unlisten1 = await listen<ConnectionEvent[]>("traffic-event", (event) => {
        addTrafficEvents(event.payload);
      });
      unlisteners.push(unlisten1);

      const unlisten2 = await listen<SpeedStats>("speed-update", (event) => {
        updateSpeedStats(event.payload);
      });
      unlisteners.push(unlisten2);

      const unlisten3 = await listen<string>("capture-status", (event) => {
        setCaptureRunning(event.payload !== "error");
      });
      unlisteners.push(unlisten3);
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
      isListening.current = false;
    };
  }, [addTrafficEvents, updateSpeedStats, setCaptureRunning]);
}
