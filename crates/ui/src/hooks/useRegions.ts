import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { AwsRegion, RegionGroup } from "../types";

export const useRegions = () => {
  const [regions, setRegions] = useState<AwsRegion[]>([]);
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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

  const loadRegions = async () => {
    setError(null);
    try {
      console.log("Loading regions...");
      const fetchedRegions = (await invoke("get_regions", {
        provider: "aws",
      })) as AwsRegion[];
      console.log({ fetchedRegions });
      setRegions(fetchedRegions);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load regions";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to load regions:", error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadRegions();
  }, []);

  useEffect(() => {
    if (regions.length > 0) {
      const grouped = groupRegionsByContinent(regions);
      setGroupedRegions(grouped);
    }
  }, [regions]);

  const clearError = () => {
    setError(null);
  };

  return {
    regions,
    groupedRegions,
    isLoading,
    error,
    loadRegions,
    clearError,
  };
};
