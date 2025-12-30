import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { AwsRegion, Instance } from "../types";

/**
 * Hook for managing EC2 instances - listing, spawning, and terminating
 */
export const useInstances = (regions: AwsRegion[]) => {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSpawning, setIsSpawning] = useState(false);
  const [isTerminating, setIsTerminating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (regions.length > 0) {
      fetchInstances();
    }
  }, [regions]);

  const fetchInstances = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const instances = await invoke<Instance[]>("list_instances");
      setInstances(instances);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to fetch instances";
      console.error("Failed to fetch existing instances:", error);
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const spawnInstance = async (regionName: string): Promise<Instance> => {
    const tempId = `spawning-${Date.now()}`;

    // Add placeholder instance in spawning state
    const spawningInstance: Instance = {
      id: tempId,
      name: "Spawning...",
      state: "spawning",
      publicIpV4: "",
      publicIpV6: "",
      region: regionName,
    };

    setInstances((prev) => [
      ...prev,
      { ...spawningInstance, region: regionName },
    ]);
    setIsSpawning(true);
    setError(null);

    try {
      const instance = await invoke<Instance>("spawn_instance", {
        region: regionName,
      });

      console.log("Server spawned:", instance);

      // Replace spawning instance with real instance
      setInstances((prev) =>
        prev.map((inst) =>
          inst.id === tempId ? { ...instance, region: regionName } : inst
        )
      );

      toast.success("Server deployed successfully!");
      return instance;
    } catch (error) {
      // Remove spawning instance on error
      setInstances((prev) => prev.filter((inst) => inst.id !== tempId));

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
      setInstances((prev) => prev.filter((i) => i.id !== instanceId));

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

  const clearError = () => {
    setError(null);
  };

  return {
    instances,
    isLoading,
    isSpawning,
    isTerminating,
    error,
    spawnInstance,
    terminateInstance,
    clearError,
    refetch: fetchInstances,
  };
};
