export function computeElapsedHours(start: string, end: string | null): number {
  const startMs = new Date(start).getTime();
  const endMs = end ? new Date(end).getTime() : Date.now();
  return Math.max(0, (endMs - startMs) / (1000 * 3600));
}
