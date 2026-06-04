import { useState } from "react";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";

export const useProfile = () => {
  const [isChecking, setIsChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkProfile = async () => {
    setIsChecking(true);
    setError(null);

    try {
      const hasProfile = (await invokeCommand("has_profile")) as boolean;
      return hasProfile;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to check profile";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to check profile:", error);
      return false;
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
