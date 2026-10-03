import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { commands } from "../bindings";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";
import { providerRegionsQueryOptions } from "../queries/providerRegions";
import toast from "react-hot-toast";
import { CloudProviderName } from "../types";

import type {
  AwsCredentials,
  AzureCredentials,
  GcpCredentials,
  OracleCredentials,
} from "../types";

type CredentialsMap = {
  [CloudProviderName.Aws]: AwsCredentials;
  [CloudProviderName.Oracle]: OracleCredentials;
  [CloudProviderName.Gcp]: GcpCredentials;
  [CloudProviderName.Azure]: AzureCredentials;
};

export function useCredentials() {
  const queryClient = useQueryClient();
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invalidateConfiguredProviders = () => {
    queryClient.invalidateQueries({
      queryKey: configuredProvidersQueryOptions.queryKey,
    });
  };

  const preloadProviderRegions = (provider: CloudProviderName) => {
    const regionsQueryOptions = providerRegionsQueryOptions(provider);
    queryClient.removeQueries({ queryKey: regionsQueryOptions.queryKey });
    queryClient.prefetchQuery(regionsQueryOptions);
  };

  const loadCredentials = async <T extends CloudProviderName>(
    provider: T,
  ): Promise<(CredentialsMap[T] & { provider: T }) | null> => {
    const result = await commands.getCredentials(provider);
    if (result.status === "error") {
      return null;
    }
    return result.data as (CredentialsMap[T] & { provider: T }) | null;
  };

  const saveCredentials = async <T extends CloudProviderName>(
    provider: T,
    credentials: CredentialsMap[T],
  ): Promise<boolean> => {
    setIsSaving(true);
    setError(null);

    const result = await commands.saveCredentials({
      provider,
      ...credentials,
    } as Parameters<typeof commands.saveCredentials>[0]);
    setIsSaving(false);
    if (result.status === "error") {
      setError(result.error);
      toast.error(result.error);
      console.error("Failed to save credentials:", result.error);
      return false;
    }
    invalidateConfiguredProviders();
    preloadProviderRegions(provider);
    return true;
  };

  const deleteCredentials = async (
    provider: CloudProviderName,
  ): Promise<boolean> => {
    const result = await commands.deleteCredentials(provider);
    if (result.status === "error") {
      toast.error(result.error);
      console.error("Failed to delete credentials:", result.error);
      return false;
    }
    invalidateConfiguredProviders();
    return true;
  };

  const clearError = () => setError(null);

  return {
    isSaving,
    error,
    loadCredentials,
    saveCredentials,
    deleteCredentials,
    clearError,
  };
}
