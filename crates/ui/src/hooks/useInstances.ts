import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { AwsRegion, ExistingInstance, ServerDetails } from "../types";

/**
 * Hook for managing EC2 instances - listing, spawning, and terminating
 */
export const useInstances = (regions: AwsRegion[]) => {
  const [existingInstances, setExistingInstances] = useState<
    ExistingInstance[]
  >([]);
  const [selectedInstance, setSelectedInstance] =
    useState<ServerDetails | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSpawning, setIsSpawning] = useState(false);
  const [isTerminating, setIsTerminating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (regions.length > 0) {
      fetchExistingInstances();
    }
  }, [regions]);

  const fetchExistingInstances = async () => {
    setIsLoading(true);
    setError(null); // Clear error at start of fetch

    try {
      // Fetch instances from all regions
      const allInstances: ExistingInstance[] = [];

      for (const region of regions) {
        try {
          const instances = await invoke<ExistingInstance[]>("list_instances", {
            region: region.name,
          });
          allInstances.push(...instances);
        } catch (error) {
          console.error(
            `Failed to fetch instances from ${region.name}:`,
            error
          );
          // Continue with other regions even if one fails
        }
      }

      console.log("Existing instances fetched successfully:", allInstances);
      console.log("Number of instances:", allInstances.length);
      setExistingInstances(allInstances);
      // Success - error stays cleared
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to fetch instances";
      console.error("Failed to fetch existing instances:", error);
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const spawnInstance = async (regionName: string): Promise<ServerDetails> => {
    setIsSpawning(true);
    setError(null);

    try {
      const result = await invoke<ServerDetails>("spawn_instance", {
        region: regionName,
      });

      console.log("Server spawned:", result);

      // Automatically add to instance list
      const newInstance: ExistingInstance = {
        id: result.instance_id,
        name: "VPN Server",
        state: "running",
        public_ip_v4: result.public_ip_v4,
        public_ip_v6: result.public_ip_v6 || "",
        region: regionName,
      };

      setExistingInstances((prev) => [...prev, newInstance]);
      setSelectedInstance(result);
      toast.success("Server deployed successfully!");

      return result;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to spawn instance";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to spawn instance:", error);
      throw error;
    } finally {
      setIsSpawning(false);
    }
  };

  const terminateInstance = async (
    instanceId: string,
    region: string
  ): Promise<void> => {
    setIsTerminating(true);
    setError(null);

    try {
      await invoke("terminate_instance", {
        instanceId,
        region,
      });

      console.log("Instance terminated:", instanceId);

      // Automatically remove from instance list
      setExistingInstances((prev) => prev.filter((i) => i.id !== instanceId));

      // Clear selection if terminated instance was selected
      if (selectedInstance?.instance_id === instanceId) {
        setSelectedInstance(null);
      }

      toast.success("Server terminated successfully!");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to terminate instance";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to terminate instance:", error);
      throw error;
    } finally {
      setIsTerminating(false);
    }
  };

  const handleInstanceSelect = (instance: ExistingInstance) => {
    const serverDetails: ServerDetails = {
      instance_id: instance.id,
      public_ip_v4: instance.public_ip_v4 || "",
      public_ip_v6: instance.public_ip_v6 || "",
      region: instance.region || "",
      client_private_key: "",
      server_public_key: "",
    };
    setSelectedInstance(serverDetails);
  };

  const clearSelectedInstance = () => {
    setSelectedInstance(null);
  };

  const clearError = () => {
    setError(null);
  };

  return {
    existingInstances,
    selectedInstance,
    isLoading,
    isSpawning,
    isTerminating,
    error,
    spawnInstance,
    terminateInstance,
    handleInstanceSelect,
    clearSelectedInstance,
    setSelectedInstance,
    clearError,
    refetch: fetchExistingInstances,
  };
};
