import { useQuery } from "@tanstack/react-query";
import { instancePricingQueryOptions } from "../queries/ledger";
import { CloudProviderName } from "../types";

const SECONDS_PER_HOUR = 3600;

export function useInstanceCost(
  provider: CloudProviderName,
  region: string,
  instanceType: string,
  uptimeSeconds: number,
): number | null {
  const { data: pricing } = useQuery({
    ...instancePricingQueryOptions(provider, region, instanceType),
    enabled: instanceType.length > 0,
  });

  if (!pricing || pricing.hourlyRate === null || pricing.ipHourlyRate === null) {
    return null;
  }

  const hourlyRate = pricing.hourlyRate + pricing.ipHourlyRate;
  return hourlyRate * (uptimeSeconds / SECONDS_PER_HOUR);
}
