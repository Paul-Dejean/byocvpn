import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

export const useCredentials = () => {
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const saveCredentials = async (
    cloudProviderName: string,
    accessKeyId: string,
    secretAccessKey: string
  ) => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await invoke("save_credentials", {
        cloudProviderName,
        accessKeyId: accessKeyId.trim(),
        secretAccessKey: secretAccessKey.trim(),
      });

      setSuccessMessage("Credentials saved successfully!");
      toast.success("Credentials saved successfully!");
      return true;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to save credentials";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to save credentials:", error);
      return false;
    } finally {
      setIsSaving(false);
    }
  };

  const clearError = () => {
    setError(null);
  };

  const clearSuccessMessage = () => {
    setSuccessMessage(null);
  };

  return {
    isSaving,
    error,
    successMessage,
    saveCredentials,
    clearError,
    clearSuccessMessage,
  };
};
