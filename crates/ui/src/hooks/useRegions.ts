import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AwsRegion, RegionGroup } from "../types";

export const useRegions = () => {
  const [regions, setRegions] = useState<AwsRegion[]>([]);
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [selectedRegion, setSelectedRegion] = useState<AwsRegion | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const groupRegionsByContinent = (regions: AwsRegion[]): RegionGroup[] => {
    const continentMap: Record<string, string> = {
      // North America
      us: "North America",
      ca: "North America",

      // Europe
      eu: "Europe",

      // Asia Pacific
      ap: "Asia Pacific",

      // South America
      sa: "South America",

      // Middle East
      me: "Middle East",

      // Africa
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

    // Convert to array and sort
    return Object.entries(groups)
      .map(([continent, regions]) => ({
        continent,
        regions: regions.sort((a, b) => a.name.localeCompare(b.name)),
      }))
      .sort((a, b) => a.continent.localeCompare(b.continent));
  };

  const loadRegions = async () => {
    try {
      console.log("Loading regions...");
      const fetchedRegions = (await invoke("get_regions")) as AwsRegion[];
      console.log({ fetchedRegions });
      setRegions(fetchedRegions);
    } catch (error) {
      console.error("Failed to load regions:", error);
    } finally {
      setIsLoading(false);
    }
  };

  // Load regions on mount
  useEffect(() => {
    loadRegions();
  }, []);

  // Group regions when they change
  useEffect(() => {
    if (regions.length > 0) {
      const grouped = groupRegionsByContinent(regions);
      setGroupedRegions(grouped);
    }
  }, [regions]);

  const handleRegionSelect = (region: AwsRegion) => {
    setSelectedRegion(region);
  };

  return {
    regions,
    groupedRegions,
    selectedRegion,
    isLoading,
    loadRegions,
    handleRegionSelect,
  };
};
