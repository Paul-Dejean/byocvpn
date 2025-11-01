import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExistingInstance, ServerDetails, AwsRegion } from "../types";

export const useInstances = (regions: AwsRegion[]) => {
  const [existingInstances, setExistingInstances] = useState<
    ExistingInstance[]
  >([]);
  const [selectedInstance, setSelectedInstance] =
    useState<ServerDetails | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const loadExistingInstances = async () => {
    setIsLoading(true);

    // Wait for regions to be loaded if they're not already
    if (regions.length === 0) {
      setIsLoading(false);
      return;
    }

    // Check all regions in parallel for existing instances
    const regionPromises = regions.map(async (region) => {
      try {
        const instances = (await invoke("list_instances", {
          region: region.name,
        })) as ExistingInstance[];

        // Add region info to each instance
        return instances.map((instance) => ({
          ...instance,
          region: region.name,
        }));
      } catch (error) {
        console.warn(`Failed to load instances from ${region.name}:`, error);
        return []; // Return empty array on error
      }
    });

    // Wait for all regions to be checked
    const allRegionResults = await Promise.all(regionPromises);

    // Flatten the results into a single array
    const allInstances = allRegionResults.flat();

    setExistingInstances(allInstances);

    // If we have a running instance, set it as the current server
    const runningInstance = allInstances.find(
      (instance) => instance.state === "running"
    );
    if (runningInstance) {
      setSelectedInstance({
        instance_id: runningInstance.id,
        public_ip_v4: runningInstance.public_ip_v4,
        public_ip_v6: runningInstance.public_ip_v6,
        region: runningInstance.region || "",
        client_private_key: "", // We don't have this from the API
        server_public_key: "", // We don't have this from the API
      });
    }

    setIsLoading(false);
  };

  // Load instances when regions are available
  useEffect(() => {
    if (regions.length > 0) {
      loadExistingInstances();
    }
  }, [regions]);

  const handleInstanceSelect = (instance: ExistingInstance) => {
    // Convert ExistingInstance to ServerDetails format
    const instanceDetails: ServerDetails = {
      instance_id: instance.id,
      public_ip_v4: instance.public_ip_v4,
      public_ip_v6: instance.public_ip_v6,
      region: instance.region || "",
      client_private_key: "", // We don't have this from the API
      server_public_key: "", // We don't have this from the API
    };

    setSelectedInstance(instanceDetails);
  };

  const addInstance = (newInstance: ExistingInstance) => {
    setExistingInstances((prev) => [...prev, newInstance]);
  };

  const removeInstance = (instanceId: string) => {
    setExistingInstances((prev) =>
      prev.filter((instance) => instance.id !== instanceId)
    );
  };

  const clearSelectedInstance = () => {
    setSelectedInstance(null);
  };

  return {
    existingInstances,
    selectedInstance,
    isLoading,
    loadExistingInstances,
    handleInstanceSelect,
    addInstance,
    removeInstance,
    clearSelectedInstance,
    setSelectedInstance,
  };
};
