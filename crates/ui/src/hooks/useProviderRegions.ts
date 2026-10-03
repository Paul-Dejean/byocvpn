import { useQuery } from "@tanstack/react-query";
import { providerRegionsQueryOptions } from "../queries/providerRegions";
import { CloudProviderName } from "../types";

export function useProviderRegions(provider: CloudProviderName) {
  const { data, isLoading } = useQuery(providerRegionsQueryOptions(provider));

  return {
    regions: data?.groupedRegions.flatMap((group) => group.regions) ?? [],
    enabledRegions: data?.enabledRegions ?? new Set<string>(),
    isLoading,
  };
}
