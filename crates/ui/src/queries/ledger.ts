import { queryOptions } from "@tanstack/react-query";
import { commands } from "../bindings";
import { AppErrorException } from "../lib/appError";
import { computeElapsedHours } from "../lib/time";
import { CloudProviderName } from "../types";
import { LedgerEntry, LedgerEntryWithCost, PricingInfo } from "../types/ledger";

const HOURS_PER_MONTH = 730;

export const ledgerQueryOptions = queryOptions({
  queryKey: ["ledger"],
  queryFn: fetchLedgerWithCosts,
});

export function instancePricingQueryOptions(
  provider: CloudProviderName,
  region: string,
  instanceType: string,
) {
  return queryOptions({
    queryKey: ["instance-pricing", provider, region, instanceType],
    queryFn: () => fetchInstancePricing(provider, region, instanceType),
    staleTime: Infinity,
  });
}

async function fetchInstancePricing(
  provider: CloudProviderName,
  region: string,
  instanceType: string,
): Promise<PricingInfo> {
  const result = await commands.getInstancePricing(provider, region, instanceType);
  if (result.status === "error") {
    throw new AppErrorException(result.error);
  }
  return result.data;
}

async function fetchLedgerWithCosts(): Promise<LedgerEntryWithCost[]> {
  const ledgerResult = await commands.getLedger();
  if (ledgerResult.status === "error") {
    throw new AppErrorException(ledgerResult.error);
  }
  return ledgerResult.data.map(computeEntryCost);
}

function computeEntryCost(entry: LedgerEntry): LedgerEntryWithCost {
  const pricing = entry.pricing ?? null;
  const isPricingUnknown =
    pricing === null ||
    pricing.hourlyRate === null ||
    pricing.ipHourlyRate === null ||
    pricing.egressRatePerGb === null ||
    pricing.storageGb === null ||
    pricing.storageRatePerGbMonth === null;
  const uptimeHours = computeElapsedHours(entry.launchedAt, entry.terminatedAt);
  const bytesSentGb = entry.bytesSent / 1024 ** 3;
  const storageGb = pricing?.storageGb ?? 0;
  const storageRatePerGbMonth = pricing?.storageRatePerGbMonth ?? 0;
  const computeCost = uptimeHours * (pricing?.hourlyRate ?? 0);
  const ipCost = uptimeHours * (pricing?.ipHourlyRate ?? 0);
  const egressCost = bytesSentGb * (pricing?.egressRatePerGb ?? 0);
  const storageCost = ((storageGb * storageRatePerGbMonth) / HOURS_PER_MONTH) * uptimeHours;
  return {
    ...entry,
    isPricingUnknown,
    estimatedCost: computeCost + ipCost + egressCost + storageCost,
    uptimeHours,
    computeCost,
    ipCost,
    egressCost,
    storageCost,
    storageGb,
    storageRatePerGbMonth,
  };
}
