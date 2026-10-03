import { useEffect, useState } from "react";

const TICK_INTERVAL_MS = 1000;

export function useInstanceUptime(launchedAt: string | null): number {
  const startTime = launchedAt ? new Date(launchedAt).getTime() : null;
  const [elapsedSeconds, setElapsedSeconds] = useState(() =>
    computeElapsedSeconds(startTime),
  );

  useEffect(() => {
    setElapsedSeconds(computeElapsedSeconds(startTime));
    if (startTime === null) {
      return;
    }
    const interval = setInterval(() => {
      setElapsedSeconds(computeElapsedSeconds(startTime));
    }, TICK_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [startTime]);

  return elapsedSeconds;
}

function computeElapsedSeconds(startTime: number | null): number {
  if (startTime === null) {
    return 0;
  }
  return Math.max(0, Math.floor((Date.now() - startTime) / 1000));
}
