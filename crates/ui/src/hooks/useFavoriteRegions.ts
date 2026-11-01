import { useQuery, useQueryClient } from "@tanstack/react-query";
import { uiPreferencesQueryOptions, updateUiPreferences } from "../queries/uiPreferences";
import { CloudProviderName } from "../types";

export function useFavoriteRegions(provider: CloudProviderName) {
  const queryClient = useQueryClient();
  const { data: preferences } = useQuery(uiPreferencesQueryOptions);
  const favoriteRegions = preferences?.favoriteRegions[provider] ?? [];

  async function toggleFavorite(regionName: string): Promise<void> {
    const next = favoriteRegions.includes(regionName)
      ? favoriteRegions.filter((name) => name !== regionName)
      : [...favoriteRegions, regionName];
    await updateUiPreferences(queryClient, {
      favoriteRegions: { ...preferences?.favoriteRegions, [provider]: next },
    });
  }

  return {
    favoriteRegions,
    isFavorite: (regionName: string) => favoriteRegions.includes(regionName),
    toggleFavorite,
  };
}
