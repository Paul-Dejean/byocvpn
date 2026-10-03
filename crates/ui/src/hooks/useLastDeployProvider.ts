import { useQuery, useQueryClient } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { CloudProviderName } from "../types";

const PROVIDERS_STORE_FILE = "providers.json";
const LAST_DEPLOY_PROVIDER_KEY = "deploy/last_provider";
const LAST_DEPLOY_PROVIDER_QUERY_KEY = ["last-deploy-provider"];

async function fetchLastDeployProvider(): Promise<CloudProviderName | null> {
  const store = await loadStore(PROVIDERS_STORE_FILE);
  const stored = await store.get<CloudProviderName>(LAST_DEPLOY_PROVIDER_KEY);
  return stored ?? null;
}

async function persistLastDeployProvider(
  provider: CloudProviderName,
): Promise<void> {
  const store = await loadStore(PROVIDERS_STORE_FILE);
  await store.set(LAST_DEPLOY_PROVIDER_KEY, provider);
  await store.save();
}

export function useLastDeployProvider(configuredProviders: CloudProviderName[]) {
  const queryClient = useQueryClient();
  const { data: storedProvider = null, isLoading } = useQuery({
    queryKey: LAST_DEPLOY_PROVIDER_QUERY_KEY,
    queryFn: fetchLastDeployProvider,
    staleTime: Infinity,
  });

  const provider =
    storedProvider !== null && configuredProviders.includes(storedProvider)
      ? storedProvider
      : (configuredProviders[0] ?? null);

  async function selectProvider(nextProvider: CloudProviderName): Promise<void> {
    queryClient.setQueryData(LAST_DEPLOY_PROVIDER_QUERY_KEY, nextProvider);
    await persistLastDeployProvider(nextProvider);
  }

  return { provider, isLoading, selectProvider };
}
