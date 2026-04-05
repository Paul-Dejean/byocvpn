import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

export interface AwsCredentials {
  accessKeyId: string;
  secretAccessKey: string;
}

export interface OracleCredentials {
  tenancyOcid: string;
  userOcid: string;
  fingerprint: string;
  privateKeyPem: string;
  region: string;
}

export interface GcpCredentials {
  projectId: string;
  serviceAccountJson: string;
}

export interface AzureCredentials {
  subscriptionId: string;
  tenantId: string;
  clientId: string;
  clientSecret: string;
}

export const useCredentials = () => {
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const loadCredentials = async (
    provider: "aws" | "oracle" | "gcp" | "azure",
  ): Promise<
    | AwsCredentials
    | OracleCredentials
    | GcpCredentials
    | AzureCredentials
    | null
  > => {
    try {
      return await invoke("get_credentials", { provider });
    } catch {
      return null;
    }
  };

  const saveCredentials = async (
    provider: "aws" | "oracle" | "gcp" | "azure",
    creds:
      | AwsCredentials
      | OracleCredentials
      | GcpCredentials
      | AzureCredentials,
  ): Promise<boolean> => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await invoke("save_credentials", { provider, creds });
      const message = "Credentials saved successfully!";
      setSuccessMessage(message);
      toast.success(message);
      return true;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to save credentials";
      setError(message);
      toast.error(message);
      console.error("Failed to save credentials:", err);
      return false;
    } finally {
      setIsSaving(false);
    }
  };

  const deleteCredentials = async (
    provider: "aws" | "oracle" | "gcp" | "azure",
  ): Promise<boolean> => {
    try {
      await invoke("delete_credentials", { provider });
      toast.success("Credentials deleted successfully!");
      return true;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to delete credentials";
      toast.error(message);
      console.error("Failed to delete credentials:", err);
      return false;
    }
  };

  const clearError = () => setError(null);
  const clearSuccessMessage = () => setSuccessMessage(null);

  return {
    isSaving,
    error,
    successMessage,
    loadCredentials,
    saveCredentials,
    deleteCredentials,
    clearError,
    clearSuccessMessage,
  };
};
