import { QueryClient, useQuery, useQueryClient } from "@tanstack/react-query";
import { isAndroid } from "../lib/platform";
import {
  UpdatePreferences,
  availableUpdateQueryOptions,
  persistCheckEnabled,
  persistDismissedVersion,
  updatePreferencesQueryOptions,
} from "../queries/updates";

export function useUpdateNotice() {
  const queryClient = useQueryClient();
  const { data: preferences } = useQuery(updatePreferencesQueryOptions);

  const isCheckActive = !isAndroid() && preferences?.checkEnabled === true;
  const { data: availableUpdate = null } = useQuery({
    ...availableUpdateQueryOptions,
    enabled: isCheckActive,
  });

  const isDismissed =
    availableUpdate !== null && preferences?.dismissedVersion === availableUpdate.version;

  async function dismissUpdate(): Promise<void> {
    if (availableUpdate === null) {
      return;
    }
    updateCachedPreferences(queryClient, { dismissedVersion: availableUpdate.version });
    await persistDismissedVersion(availableUpdate.version);
  }

  async function setCheckEnabled(checkEnabled: boolean): Promise<void> {
    updateCachedPreferences(queryClient, { checkEnabled });
    await persistCheckEnabled(checkEnabled);
  }

  return {
    availableUpdate: isCheckActive && !isDismissed ? availableUpdate : null,
    isCheckEnabled: preferences?.checkEnabled ?? true,
    dismissUpdate,
    setCheckEnabled,
  };
}

function updateCachedPreferences(queryClient: QueryClient, changes: Partial<UpdatePreferences>) {
  queryClient.setQueryData<UpdatePreferences>(
    updatePreferencesQueryOptions.queryKey,
    (current) => ({
      checkEnabled: true,
      dismissedVersion: null,
      ...current,
      ...changes,
    }),
  );
}
