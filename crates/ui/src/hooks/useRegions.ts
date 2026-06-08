import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";
import { CloudProviderName, Region, RegionGroup } from "../types";

const REGION_PREFIX_TO_CONTINENT: Record<string, string> = {
  us: "North America",
  ca: "North America",
  eu: "Europe",
  ap: "Asia Pacific",
  sa: "South America",
  me: "Middle East",
  af: "Africa",
};

interface RegionsData {
  regions: Region[];
  groupedRegions: RegionGroup[];
}

export function useRegions() {
  const queryClient = useQueryClient();
  const { data, isLoading, error } = useQuery({
    queryKey: ["all-regions"],
    queryFn: fetchAllRegions,
    staleTime: 30_000,
  });

  useEffect(() => {
    if (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to load regions",
      );
    }
  }, [error]);

  return {
    regions: data?.regions ?? [],
    groupedRegions: data?.groupedRegions ?? [],
    isLoading,
    error: error instanceof Error ? error.message : null,
    loadRegions: () =>
      queryClient.invalidateQueries({ queryKey: ["all-regions"] }),
    clearError: () => {},
  };
}

async function fetchAllRegions(): Promise<RegionsData> {
  const configuredProviders = await fetchConfiguredProviders();
  if (configuredProviders.length === 0) {
    return { regions: [], groupedRegions: [] };
  }
  const primaryProvider = configuredProviders[0];
  const fetchedRegions = await invokeCommand<Region[]>("get_regions", {
    provider: primaryProvider,
  });
  return {
    regions: fetchedRegions,
    groupedRegions: groupRegionsByContinent(fetchedRegions),
  };
}

async function fetchConfiguredProviders(): Promise<CloudProviderName[]> {
  const checks = await Promise.all(
    Object.values(CloudProviderName).map(async (provider) => {
      try {
        const credentials = await invokeCommand("get_credentials", {
          provider,
        });
        return credentials !== null ? provider : null;
      } catch {
        return null;
      }
    }),
  );
  return checks.filter(
    (provider): provider is CloudProviderName => provider !== null,
  );
}

function groupRegionsByContinent(regions: Region[]): RegionGroup[] {
  const groups: Record<string, Region[]> = {};
  regions.forEach((region) => {
    const prefix = region.name.split("-")[0];
    const continent = REGION_PREFIX_TO_CONTINENT[prefix] ?? "Other";
    if (!groups[continent]) {
      groups[continent] = [];
    }
    groups[continent].push(region);
  });
  return Object.entries(groups)
    .map(([continent, continentRegions]) => ({
      continent,
      regions: continentRegions.sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .sort((a, b) => a.continent.localeCompare(b.continent));
}
