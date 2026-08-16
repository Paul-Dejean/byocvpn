import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { fetchConfiguredProviders } from "../lib/fetchConfiguredProviders";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";
import { Region, RegionGroup } from "../types";

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
    refetchOnReconnect: false,
  });

  useEffect(() => {
    if (error) {
      toast.error(extractErrorMessage(error, "Failed to load regions"));
    }
  }, [error]);

  return {
    regions: data?.regions ?? [],
    groupedRegions: data?.groupedRegions ?? [],
    isLoading,
    error: error ? extractErrorMessage(error, "Failed to load regions") : null,
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
