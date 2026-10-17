import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { commands } from "../bindings";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";
import { providerRegionsQueryOptions } from "../queries/providerRegions";
import { toast } from "react-hot-toast";
import { CloudProviderName } from "../types";

import type { ProviderCredentials } from "../types";

type ProviderCredentialsFor<T extends CloudProviderName> = Extract<
  ProviderCredentials,
  { provider: T }
>;

export function useCredentials() {
  const queryClient = useQueryClient();
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invalidateConfiguredProviders = () => {
    void queryClient.invalidateQueries({
      queryKey: configuredProvidersQueryOptions.queryKey,
    });
  };

  const preloadProviderRegions = (provider: CloudProviderName) => {
    const regionsQueryOptions = providerRegionsQueryOptions(provider);
    queryClient.removeQueries({ queryKey: regionsQueryOptions.queryKey });
    void queryClient.prefetchQuery(regionsQueryOptions);
  };

  const saveCredentials = async (credentials: ProviderCredentials): Promise<boolean> => {
    setIsSaving(true);
    setError(null);

    const result = await commands.saveCredentials(credentials);
    setIsSaving(false);
    if (result.status === "error") {
      setError(result.error);
      toast.error(result.error);
      console.error("Failed to save credentials:", result.error);
      return false;
    }
    invalidateConfiguredProviders();
    preloadProviderRegions(credentials.provider);
    return true;
  };

  const deleteCredentials = async (provider: CloudProviderName): Promise<boolean> => {
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

async function loadCredentials<T extends CloudProviderName>(
  provider: T,
): Promise<ProviderCredentialsFor<T> | null> {
  const result = await commands.getCredentials(provider);
  if (result.status === "error") {
    return null;
  }
  const credentials = result.data;
  if (credentials === null || !isCredentialsForProvider(credentials, provider)) {
    return null;
  }
  return credentials;
}

function isCredentialsForProvider<T extends CloudProviderName>(
  credentials: ProviderCredentials,
  provider: T,
): credentials is ProviderCredentialsFor<T> {
  return credentials.provider === provider;
}
