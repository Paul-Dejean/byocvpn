import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { AwsRegion, RegionGroup } from "../types";

const PROVIDERS = ["aws", "oracle", "gcp", "azure"] as const;

type Provider = (typeof PROVIDERS)[number];

const groupRegionsByContinent = (regions: AwsRegion[]): RegionGroup[] => {
  const continentMap: Record<string, string> = {
    us: "North America",
    ca: "North America",
    eu: "Europe",
    ap: "Asia Pacific",
    sa: "South America",
    me: "Middle East",
    af: "Africa",
  };

  const groups: Record<string, AwsRegion[]> = {};

  regions.forEach((region) => {
    const prefix = region.name.split("-")[0];
    const continent = continentMap[prefix] || "Other";
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
};

const fetchConfiguredProviders = async (): Promise<Provider[]> => {
  const checks = await Promise.all(
    PROVIDERS.map(async (provider) => {
      try {
        const credentials = await invoke("get_credentials", { provider });
        return credentials !== null ? provider : null;
      } catch {
        return null;
      }
    }),
  );
  return checks.filter((provider): provider is Provider => provider !== null);
};

export const useRegions = () => {
  const [regions, setRegions] = useState<AwsRegion[]>([]);
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
      const fetchedRegions = (await invoke("get_regions", {
        provider: primaryProvider,
      })) as AwsRegion[];
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
};
