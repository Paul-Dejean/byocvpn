import { useQuery, useQueryClient } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { CloudProviderName } from "../types";

const FAVORITES_STORE_FILE = "providers.json";

function buildFavoritesKey(provider: CloudProviderName): string {
  return `favorite_regions/${provider}`;
}

async function fetchFavoriteRegions(
  provider: CloudProviderName,
): Promise<string[]> {
  const store = await loadStore(FAVORITES_STORE_FILE);
  const stored = await store.get<string[]>(buildFavoritesKey(provider));
  return stored ?? [];
}

async function persistFavoriteRegions(
  provider: CloudProviderName,
  favorites: string[],
): Promise<void> {
  const store = await loadStore(FAVORITES_STORE_FILE);
  await store.set(buildFavoritesKey(provider), favorites);
  await store.save();
}

export function useFavoriteRegions(provider: CloudProviderName) {
  const queryClient = useQueryClient();
  const queryKey = ["favorite-regions", provider];
  const { data: favoriteRegions = [] } = useQuery({
    queryKey,
    queryFn: () => fetchFavoriteRegions(provider),
    staleTime: Infinity,
  });

  async function toggleFavorite(regionName: string): Promise<void> {
    const next = favoriteRegions.includes(regionName)
      ? favoriteRegions.filter((name) => name !== regionName)
      : [...favoriteRegions, regionName];
    queryClient.setQueryData(queryKey, next);
    await persistFavoriteRegions(provider, next);
  }

  return {
    favoriteRegions,
    isFavorite: (regionName: string) => favoriteRegions.includes(regionName),
    toggleFavorite,
  };
}
