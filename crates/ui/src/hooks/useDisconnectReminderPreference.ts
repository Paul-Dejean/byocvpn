import { useQuery, useQueryClient } from "@tanstack/react-query";
import { uiPreferencesQueryOptions, updateUiPreferences } from "../queries/uiPreferences";

const MAX_DISMISSALS_BEFORE_MUTING = 3;

export function useDisconnectReminderPreference() {
  const queryClient = useQueryClient();
  const { data: preferences, isLoading } = useQuery(uiPreferencesQueryOptions);
  const dismissalCount = preferences?.disconnectReminderDismissalCount ?? 0;

  const isMuted = isLoading || dismissalCount >= MAX_DISMISSALS_BEFORE_MUTING;

  async function updateDismissalCount(nextDismissalCount: number): Promise<void> {
    await updateUiPreferences(queryClient, {
      disconnectReminderDismissalCount: nextDismissalCount,
    });
  }

  async function recordDismissal(): Promise<void> {
    await updateDismissalCount(dismissalCount + 1);
  }

  async function muteReminder(): Promise<void> {
    await updateDismissalCount(MAX_DISMISSALS_BEFORE_MUTING);
  }

  return { isMuted, recordDismissal, muteReminder };
}
