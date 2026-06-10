import { useState } from "react";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";
import { CloudProviderName } from "../types";

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
  applicationId: string;
  secretValue: string;
}

type CredentialsMap = {
  [CloudProviderName.Aws]: AwsCredentials;
  [CloudProviderName.Oracle]: OracleCredentials;
  [CloudProviderName.Gcp]: GcpCredentials;
  [CloudProviderName.Azure]: AzureCredentials;
};

export function useCredentials() {
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const loadCredentials = async <T extends CloudProviderName>(
    provider: T,
  ): Promise<(CredentialsMap[T] & { provider: T }) | null> => {
    try {
      return await invokeCommand("get_credentials", { provider });
    } catch {
      return null;
    }
  };

  const saveCredentials = async <T extends CloudProviderName>(
    provider: T,
    credentials: CredentialsMap[T],
  ): Promise<boolean> => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await invokeCommand("save_credentials", { credentials: { provider, ...credentials } });
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
    provider: CloudProviderName,
  ): Promise<boolean> => {
    try {
      await invokeCommand("delete_credentials", { provider });
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
}
