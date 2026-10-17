import { queryOptions } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { commands } from "../bindings";
import { CloudProviderName, Region, RegionGroup } from "../types";

export interface ProviderRegionsData {
  groupedRegions: RegionGroup[];
  enabledRegions: Set<string>;
}

export function providerRegionsQueryOptions(provider: CloudProviderName) {
  return queryOptions({
    queryKey: ["regions", provider],
    queryFn: () => fetchProviderRegions(provider),
    staleTime: 30_000,
  });
}

async function fetchProviderRegions(provider: CloudProviderName): Promise<ProviderRegionsData> {
  const regionsResult = await commands.getRegions(provider);
  if (regionsResult.status === "error") {
    throw new Error(regionsResult.error);
  }
  const regions = regionsResult.data;
  const store = await loadStore("providers.json");
  const regionFlags = await Promise.all(
    regions.map(async (region) => ({
      name: region.name,
      enabled: await store.get<boolean>(`enabled_regions/${provider}/${region.name}`),
    })),
  );
  const enabledRegions = new Set<string>(
    regionFlags
      .filter((regionFlag) => regionFlag.enabled === true)
      .map((regionFlag) => regionFlag.name),
  );
  return { groupedRegions: groupRegionsByCountry(regions), enabledRegions };
}

function groupRegionsByCountry(regions: Region[]): RegionGroup[] {
  const groups: Record<string, Region[]> = {};
  for (const region of regions) {
    if (!groups[region.country]) groups[region.country] = [];
    groups[region.country].push(region);
  }
  return Object.entries(groups)
    .map(([continent, continentRegions]) => ({
      continent,
      regions: continentRegions.toSorted((a, b) => a.name.localeCompare(b.name)),
    }))
    .toSorted((a, b) => a.continent.localeCompare(b.continent));
}
