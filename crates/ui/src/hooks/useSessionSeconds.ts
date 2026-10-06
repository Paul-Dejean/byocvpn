import { useEffect, useState } from "react";

const TICK_INTERVAL_MS = 1000;

export function useSessionSeconds(connectedAt: number | null): number {
  const [fallbackStartTime] = useState(() => Date.now());
  const startTime = connectedAt !== null ? connectedAt * 1000 : fallbackStartTime;
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  useEffect(() => {
    function updateElapsedSeconds() {
      setElapsedSeconds(Math.max(0, Math.floor((Date.now() - startTime) / 1000)));
    }
    updateElapsedSeconds();
    const interval = setInterval(updateElapsedSeconds, TICK_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [startTime]);

  return elapsedSeconds;
}
