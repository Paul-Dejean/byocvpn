import { QueryClient, queryOptions } from "@tanstack/react-query";
import { commands } from "../bindings";
import { computeElapsedHours } from "../lib/time";
import { CloudProviderName } from "../types";
import { LedgerEntry, LedgerEntryWithCost, PricingInfo } from "../types/ledger";

export function ledgerQueryOptions(queryClient: QueryClient) {
  return queryOptions({
    queryKey: ["ledger"],
    queryFn: () => fetchEnrichedLedger(queryClient),
  });
}

export function instancePricingQueryOptions(
  provider: CloudProviderName,
  instanceType: string,
) {
  return queryOptions({
    queryKey: ["instance-pricing", provider, instanceType],
    queryFn: () => fetchInstancePricing(provider, instanceType),
    staleTime: Infinity,
  });
}

async function fetchInstancePricing(
  provider: CloudProviderName,
  instanceType: string,
): Promise<PricingInfo> {
  const result = await commands.getInstancePricing(provider, instanceType);
  if (result.status === "error") {
    throw new Error(result.error);
  }
  return result.data;
}

async function fetchEnrichedLedger(
  queryClient: QueryClient,
): Promise<LedgerEntryWithCost[]> {
  const ledgerResult = await commands.getLedger();
  if (ledgerResult.status === "error") {
    throw new Error(ledgerResult.error);
  }
  const ledgerEntries = ledgerResult.data;
  return enrichLedgerEntriesWithCosts(ledgerEntries, queryClient);
}

async function enrichLedgerEntriesWithCosts(
  ledgerEntries: LedgerEntry[],
  queryClient: QueryClient,
): Promise<LedgerEntryWithCost[]> {
  return Promise.all(
    ledgerEntries.map(async (entry) => {
      const pricing = await resolveInstancePricing(entry, queryClient);
      const isPricingUnknown =
        pricing === null ||
        pricing.hourlyRate === null ||
        pricing.ipHourlyRate === null ||
        pricing.egressRatePerGb === null ||
        pricing.storageGb === null ||
        pricing.storageRatePerGbMonth === null;
      const uptimeHours = computeElapsedHours(
        entry.launchedAt,
        entry.terminatedAt,
      );
      const bytesSentGb = entry.bytesSent / 1024 ** 3;
      const hourlyRate = pricing?.hourlyRate ?? 0;
      const ipHourlyRate = pricing?.ipHourlyRate ?? 0;
      const egressRatePerGb = pricing?.egressRatePerGb ?? 0;
      const storageGb = pricing?.storageGb ?? 0;
      const storageRatePerGbMonth = pricing?.storageRatePerGbMonth ?? 0;
      const computeCost = uptimeHours * hourlyRate;
      const ipCost = uptimeHours * ipHourlyRate;
      const egressCost = bytesSentGb * egressRatePerGb;
      const storageCost =
        ((storageGb * storageRatePerGbMonth) / 730) * uptimeHours;
      const estimatedCost = computeCost + ipCost + egressCost + storageCost;
      return {
        ...entry,
        isPricingUnknown,
        estimatedCost,
        uptimeHours,
        computeCost,
        ipCost,
        egressCost,
        storageCost,
        storageGb,
        storageRatePerGbMonth,
      };
    }),
  );
}

async function resolveInstancePricing(
  entry: LedgerEntry,
  queryClient: QueryClient,
): Promise<PricingInfo | null> {
  try {
    return await queryClient.fetchQuery(
      instancePricingQueryOptions(entry.provider, entry.instanceType),
    );
  } catch {
    return null;
  }
}
