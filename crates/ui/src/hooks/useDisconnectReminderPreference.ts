import { useQuery, useQueryClient } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";

const REMINDERS_STORE_FILE = "providers.json";
const DISMISSAL_COUNT_KEY = "reminders/disconnect_dismissal_count";
const DISMISSAL_COUNT_QUERY_KEY = ["disconnect-reminder-dismissal-count"];
const MAX_DISMISSALS_BEFORE_MUTING = 3;

export function useDisconnectReminderPreference() {
  const queryClient = useQueryClient();
  const { data: dismissalCount = 0, isLoading } = useQuery({
    queryKey: DISMISSAL_COUNT_QUERY_KEY,
    queryFn: fetchDismissalCount,
    staleTime: Infinity,
  });

  const isMuted = isLoading || dismissalCount >= MAX_DISMISSALS_BEFORE_MUTING;

  async function updateDismissalCount(nextDismissalCount: number): Promise<void> {
    queryClient.setQueryData(DISMISSAL_COUNT_QUERY_KEY, nextDismissalCount);
    await persistDismissalCount(nextDismissalCount);
  }

  async function recordDismissal(): Promise<void> {
    await updateDismissalCount(dismissalCount + 1);
  }

  async function muteReminder(): Promise<void> {
    await updateDismissalCount(MAX_DISMISSALS_BEFORE_MUTING);
  }

  return { isMuted, recordDismissal, muteReminder };
}

async function fetchDismissalCount(): Promise<number> {
  const store = await loadStore(REMINDERS_STORE_FILE);
  const stored = await store.get<number>(DISMISSAL_COUNT_KEY);
  return stored ?? 0;
}

async function persistDismissalCount(dismissalCount: number): Promise<void> {
  const store = await loadStore(REMINDERS_STORE_FILE);
  await store.set(DISMISSAL_COUNT_KEY, dismissalCount);
  await store.save();
}
