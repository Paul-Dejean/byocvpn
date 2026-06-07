import { useState, useEffect } from "react";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { invokeCommand } from "../lib/invokeCommand";
import { CloudProviderName, Region, RegionGroup } from "../types";

function groupRegionsByCountry(regions: Region[]): RegionGroup[] {
  const groups: Record<string, Region[]> = {};
  for (const region of regions) {
    if (!groups[region.country]) groups[region.country] = [];
    groups[region.country].push(region);
  }
  return Object.entries(groups)
    .map(([continent, continentRegions]) => ({
      continent,
      regions: continentRegions.sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .sort((a, b) => a.continent.localeCompare(b.continent));
}

async function fetchEnabledRegions(provider: CloudProviderName, regions: Region[]): Promise<Set<string>> {
  const store = await loadStore("providers.json");
  const enabled = new Set<string>();
  for (const region of regions) {
    const value = await store.get<boolean>(`enabled_regions/${provider}/${region.name}`);
    if (value === true) enabled.add(region.name);
  }
  return enabled;
}

export function useProviderRegions(provider: CloudProviderName) {
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [enabledRegions, setEnabledRegions] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    setIsLoading(true);
    setGroupedRegions([]);
    setEnabledRegions(new Set());

    invokeCommand<Region[]>("get_regions", { provider })
      .then(async (regions) => {
        setGroupedRegions(groupRegionsByCountry(regions));
        setEnabledRegions(await fetchEnabledRegions(provider, regions));
      })
      .catch(console.error)
      .finally(() => setIsLoading(false));
  }, [provider]);

  const markRegionEnabled = (regionName: string) => {
    setEnabledRegions((previous) => new Set([...previous, regionName]));
  };

  return { groupedRegions, enabledRegions, isLoading, markRegionEnabled };
}
