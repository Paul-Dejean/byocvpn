import { useEffect, useState } from "react";

const TICK_INTERVAL_MS = 1000;

export function useInstanceUptime(launchedAt: string | null): number {
  const startTime = launchedAt ? new Date(launchedAt).getTime() : null;
  const [elapsedSeconds, setElapsedSeconds] = useState(() => computeElapsedSeconds(startTime));
  const [previousStartTime, setPreviousStartTime] = useState(startTime);

  if (previousStartTime !== startTime) {
    setPreviousStartTime(startTime);
    setElapsedSeconds(computeElapsedSeconds(startTime));
  }

  useEffect(() => {
    if (startTime === null) {
      return undefined;
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
