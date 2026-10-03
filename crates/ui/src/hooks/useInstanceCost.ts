import { useQuery } from "@tanstack/react-query";
import { instancePricingQueryOptions } from "../queries/ledger";
import { CloudProviderName } from "../types";

const SECONDS_PER_HOUR = 3600;

export function useInstanceCost(
  provider: CloudProviderName,
  instanceType: string,
  uptimeSeconds: number,
): number | null {
  const { data: pricing } = useQuery({
    ...instancePricingQueryOptions(provider, instanceType),
    enabled: instanceType.length > 0,
  });

  if (!pricing || pricing.hourlyRate === null) {
    return null;
  }

  const hourlyRate = pricing.hourlyRate + (pricing.ipHourlyRate ?? 0);
  return hourlyRate * (uptimeSeconds / SECONDS_PER_HOUR);
}
