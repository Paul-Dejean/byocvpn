import { QueryClient, useQuery, useQueryClient } from "@tanstack/react-query";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { invokeCommand } from "../lib/invokeCommand";
import { computeElapsedHours } from "../lib/time";
import { LedgerEntry, LedgerEntryWithCost, PricingInfo } from "../types/ledger";

export function useLedger() {
  const queryClient = useQueryClient();
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["ledger"],
    queryFn: () => fetchEnrichedLedger(queryClient),
  });

  return {
    entries: data ?? [],
    isLoading,
    error: error
      ? extractErrorMessage(error, "Failed to load cost ledger")
      : null,
    refetch: () => {
      refetch();
    },
  };
}

async function fetchEnrichedLedger(
  queryClient: QueryClient,
): Promise<LedgerEntryWithCost[]> {
  const ledgerEntries = await invokeCommand<LedgerEntry[]>("get_ledger");
  return enrichLedgerEntriesWithCosts(ledgerEntries, queryClient);
}

async function enrichLedgerEntriesWithCosts(
  ledgerEntries: LedgerEntry[],
  queryClient: QueryClient,
): Promise<LedgerEntryWithCost[]> {
  return Promise.all(
    ledgerEntries.map(async (entry) => {
      const pricing = await fetchInstancePricing(entry, queryClient);
      const uptimeHours = computeElapsedHours(
        entry.launchedAt,
        entry.terminatedAt,
      );
      const bytesSentGb = entry.bytesSent / 1024 ** 3;
      const computeCost = uptimeHours * pricing.hourlyRate;
      const ipCost = uptimeHours * pricing.ipHourlyRate;
      const egressCost = bytesSentGb * pricing.egressRatePerGb;
      const storageCost =
        ((pricing.storageGb * pricing.storageRatePerGbMonth) / 730) *
        uptimeHours;
      const estimatedCost = computeCost + ipCost + egressCost + storageCost;
      return {
        ...entry,
        estimatedCost,
        uptimeHours,
        computeCost,
        ipCost,
        egressCost,
        storageCost,
        storageGb: pricing.storageGb,
        storageRatePerGbMonth: pricing.storageRatePerGbMonth,
      };
    }),
  );
}

async function fetchInstancePricing(
  entry: LedgerEntry,
  queryClient: QueryClient,
): Promise<PricingInfo> {
  try {
    return await queryClient.fetchQuery({
      queryKey: ["instance-pricing", entry.provider, entry.instanceType],
      queryFn: () =>
        invokeCommand<PricingInfo>("get_instance_pricing", {
          provider: entry.provider,
          instanceType: entry.instanceType,
        }),
      staleTime: Infinity,
    });
  } catch {
    return ZERO_PRICING;
  }
}

const ZERO_PRICING: PricingInfo = {
  hourlyRate: 0,
  ipHourlyRate: 0,
  egressRatePerGb: 0,
  storageGb: 0,
  storageRatePerGbMonth: 0,
};
