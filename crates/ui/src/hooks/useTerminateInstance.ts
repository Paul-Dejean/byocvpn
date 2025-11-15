import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

export const useTerminateInstance = () => {
  const [isTerminating, setIsTerminating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const terminateInstance = async (instanceId: string, region: string) => {
    setIsTerminating(true);
    setError(null);

    try {
      const result = await invoke("terminate_instance", {
        instanceId,
        region,
      });

      console.log("Server terminated:", result);
      return result;
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
    isTerminating,
    error,
    terminateInstance,
    clearError,
  };
};
