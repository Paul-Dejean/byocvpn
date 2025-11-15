import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { ServerDetails } from "../types";

export const useSpawnInstance = () => {
  const [isSpawning, setIsSpawning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const spawnInstance = async (regionName: string) => {
    setIsSpawning(true);
    setError(null);

    try {
      const result = await invoke("spawn_instance", {
        region: regionName,
      });

      console.log("Server spawned:", result);
      const serverDetails = result as ServerDetails;
      return serverDetails;
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

  const clearError = () => {
    setError(null);
  };

  return {
    isSpawning,
    error,
    spawnInstance,
    clearError,
  };
};
