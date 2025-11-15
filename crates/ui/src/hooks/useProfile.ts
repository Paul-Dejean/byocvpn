import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

export const useProfile = () => {
  const [isChecking, setIsChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkProfile = async () => {
    setIsChecking(true);
    setError(null);

    try {
      const hasProfile = (await invoke("has_profile")) as boolean;
      return hasProfile;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to check profile";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to check profile:", error);
      return false; // Default to no profile on error
    } finally {
      setIsChecking(false);
    }
  };

  const clearError = () => {
    setError(null);
  };

  return {
    isChecking,
    error,
    checkProfile,
    clearError,
  };
};
