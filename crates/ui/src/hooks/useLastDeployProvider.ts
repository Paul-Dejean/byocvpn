import { useQuery, useQueryClient } from "@tanstack/react-query";
import { uiPreferencesQueryOptions, updateUiPreferences } from "../queries/uiPreferences";
import { CloudProviderName } from "../types";

export function useLastDeployProvider(configuredProviders: CloudProviderName[]) {
  const queryClient = useQueryClient();
  const { data: preferences, isLoading } = useQuery(uiPreferencesQueryOptions);
  const storedProvider = preferences?.lastDeployProvider ?? null;

  const provider =
    storedProvider !== null && configuredProviders.includes(storedProvider)
      ? storedProvider
      : (configuredProviders[0] ?? null);

  async function selectProvider(nextProvider: CloudProviderName): Promise<void> {
    await updateUiPreferences(queryClient, { lastDeployProvider: nextProvider });
  }

  return { provider, isLoading, selectProvider };
}
