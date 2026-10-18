import { queryOptions } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { isNewerVersion } from "../lib/version";

const UPDATES_STORE_FILE = "updates.json";
const CHECK_ENABLED_KEY = "updates/check_enabled";
const DISMISSED_VERSION_KEY = "updates/dismissed_version";
const LATEST_RELEASE_API_URL = "https://api.github.com/repos/Paul-Dejean/byocvpn/releases/latest";
const RELEASE_PAGE_URL_PREFIX = "https://github.com/Paul-Dejean/byocvpn/";
const RELEASE_CHECK_STALE_TIME_MS = 6 * 60 * 60 * 1000;

export interface AvailableUpdate {
  version: string;
  releaseUrl: string;
}

export interface UpdatePreferences {
  checkEnabled: boolean;
  dismissedVersion: string | null;
}

interface LatestReleaseResponse {
  tag_name: string;
  html_url: string;
}

export const updatePreferencesQueryOptions = queryOptions({
  queryKey: ["update-preferences"],
  queryFn: fetchUpdatePreferences,
  staleTime: Infinity,
});

export const availableUpdateQueryOptions = queryOptions({
  queryKey: ["available-update"],
  queryFn: fetchAvailableUpdate,
  staleTime: RELEASE_CHECK_STALE_TIME_MS,
  retry: false,
  refetchOnWindowFocus: false,
});

export async function persistCheckEnabled(checkEnabled: boolean): Promise<void> {
  const store = await loadStore(UPDATES_STORE_FILE);
  await store.set(CHECK_ENABLED_KEY, checkEnabled);
  await store.save();
}

export async function persistDismissedVersion(dismissedVersion: string): Promise<void> {
  const store = await loadStore(UPDATES_STORE_FILE);
  await store.set(DISMISSED_VERSION_KEY, dismissedVersion);
  await store.save();
}

async function fetchUpdatePreferences(): Promise<UpdatePreferences> {
  const store = await loadStore(UPDATES_STORE_FILE);
  const checkEnabled = await store.get<boolean>(CHECK_ENABLED_KEY);
  const dismissedVersion = await store.get<string>(DISMISSED_VERSION_KEY);
  return {
    checkEnabled: checkEnabled ?? true,
    dismissedVersion: dismissedVersion ?? null,
  };
}

async function fetchAvailableUpdate(): Promise<AvailableUpdate | null> {
  const [currentVersion, response] = await Promise.all([
    getVersion(),
    fetch(LATEST_RELEASE_API_URL, { headers: { Accept: "application/vnd.github+json" } }),
  ]);

  if (!response.ok) {
    throw new Error(`GitHub responded with status ${response.status}`);
  }

  const latestRelease: unknown = await response.json();
  if (!isLatestReleaseResponse(latestRelease)) {
    throw new Error("Unexpected response from GitHub");
  }

  const isReleaseLinkTrusted = latestRelease.html_url.startsWith(RELEASE_PAGE_URL_PREFIX);

  if (!isReleaseLinkTrusted || !isNewerVersion(latestRelease.tag_name, currentVersion)) {
    return null;
  }

  return {
    version: latestRelease.tag_name.replace(/^v/, ""),
    releaseUrl: latestRelease.html_url,
  };
}

function isLatestReleaseResponse(payload: unknown): payload is LatestReleaseResponse {
  return (
    typeof payload === "object" &&
    payload !== null &&
    "tag_name" in payload &&
    typeof payload.tag_name === "string" &&
    "html_url" in payload &&
    typeof payload.html_url === "string"
  );
}
