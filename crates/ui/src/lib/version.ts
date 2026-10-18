type StableVersion = [major: number, minor: number, patch: number];

const STABLE_VERSION_PATTERN = /^v?(\d+)\.(\d+)\.(\d+)$/;

export function isNewerVersion(candidateVersion: string, currentVersion: string): boolean {
  const candidate = parseStableVersion(candidateVersion);
  const current = parseStableVersion(currentVersion);
  if (candidate === null || current === null) {
    return false;
  }

  const [candidateMajor, candidateMinor, candidatePatch] = candidate;
  const [currentMajor, currentMinor, currentPatch] = current;

  if (candidateMajor !== currentMajor) {
    return candidateMajor > currentMajor;
  }
  if (candidateMinor !== currentMinor) {
    return candidateMinor > currentMinor;
  }
  return candidatePatch > currentPatch;
}

function parseStableVersion(version: string): StableVersion | null {
  const match = STABLE_VERSION_PATTERN.exec(version.trim());
  if (match === null) {
    return null;
  }
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}
