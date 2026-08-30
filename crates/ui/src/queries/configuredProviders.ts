import { queryOptions } from "@tanstack/react-query";
import { commands } from "../bindings";
import { CloudProviderName } from "../types";

export const configuredProvidersQueryOptions = queryOptions({
  queryKey: ["configured-providers"],
  queryFn: fetchConfiguredProviders,
  staleTime: 30_000,
});

async function fetchConfiguredProviders(): Promise<CloudProviderName[]> {
  const checks = await Promise.all(
    Object.values(CloudProviderName).map(async (provider) => {
      const result = await commands.getCredentials(provider);
      if (result.status === "error") {
        return null;
      }
      return result.data !== null ? provider : null;
    }),
  );
  return checks.filter(
    (provider): provider is CloudProviderName => provider !== null,
  );
}
