import { useState, useEffect } from "react";
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
    .map(([continent, regions]) => ({
      continent,
      regions: regions.sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .sort((a, b) => a.continent.localeCompare(b.continent));
}

async function fetchConfiguredProviders(): Promise<CloudProviderName[]> {
  const checks = await Promise.all(
    Object.values(CloudProviderName).map(async (provider) => {
      try {
        const credentials = await invokeCommand("get_credentials", { provider });
        return credentials !== null ? provider : null;
      } catch {
        return null;
      }
    }),
  );
  return checks.filter((provider): provider is CloudProviderName => provider !== null);
}

export function useRegions() {
  const [regions, setRegions] = useState<Region[]>([]);
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadRegions = async () => {
    setError(null);
    try {
      const configuredProviders = await fetchConfiguredProviders();
      if (configuredProviders.length === 0) {
        setIsLoading(false);
        return;
      }

      const primaryProvider = configuredProviders[0];
      const fetchedRegions = (await invokeCommand("get_regions", {
        provider: primaryProvider,
      })) as Region[];
      setRegions(fetchedRegions);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load regions";
      setError(errorMessage);
      toast.error(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadRegions();
  }, []);

  useEffect(() => {
    if (regions.length > 0) {
      setGroupedRegions(groupRegionsByContinent(regions));
    }
  }, [regions]);

  const clearError = () => setError(null);

  return {
    regions,
    groupedRegions,
    isLoading,
    error,
    loadRegions,
    clearError,
  };
}
