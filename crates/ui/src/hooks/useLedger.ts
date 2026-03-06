import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LedgerEntry, LedgerEntryWithCost, PricingInfo } from "../types/ledger";

function computeUptimeHours(
  launchedAt: string,
  terminatedAt: string | null,
): number {
  const start = new Date(launchedAt).getTime();
  const end = terminatedAt ? new Date(terminatedAt).getTime() : Date.now();
  return Math.max(0, (end - start) / (1000 * 3600));
}

function computeEstimatedCost(
  entry: LedgerEntry,
  pricing: PricingInfo,
  uptimeHours: number,
): number {
  const bytesSentGb = entry.bytesSent / 1024 ** 3;
  return (
    uptimeHours * (pricing.hourlyRate + pricing.ipHourlyRate) +
    bytesSentGb * pricing.egressRatePerGb
  );
}

/**
 * Fetches all ledger entries, resolves per-entry pricing, and computes
 * estimated costs. Returns enriched entries ready for display.
 */
export const useLedger = () => {
  const [entries, setEntries] = useState<LedgerEntryWithCost[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchLedger = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const rawEntries = await invoke<LedgerEntry[]>("get_ledger");

      // Resolve pricing for each unique (provider, instanceType) pair once.
      const pricingCache = new Map<string, PricingInfo>();
      for (const entry of rawEntries) {
        const key = `${entry.provider}::${entry.instanceType}`;
        if (!pricingCache.has(key)) {
          try {
            const pricing = await invoke<PricingInfo>("get_instance_pricing", {
              provider: entry.provider,
              instanceType: entry.instanceType,
            });
            pricingCache.set(key, pricing);
          } catch {
            // Unknown pricing — treat as free so the entry is still shown.
            pricingCache.set(key, {
              hourlyRate: 0,
              ipHourlyRate: 0,
              egressRatePerGb: 0,
            });
          }
        }
      }

      const enriched: LedgerEntryWithCost[] = rawEntries.map((entry) => {
        const key = `${entry.provider}::${entry.instanceType}`;
        const pricing = pricingCache.get(key)!;
        const uptimeHours = computeUptimeHours(
          entry.launchedAt,
          entry.terminatedAt,
        );
        const estimatedCost = computeEstimatedCost(entry, pricing, uptimeHours);
        return { ...entry, estimatedCost, uptimeHours };
      });

      setEntries(enriched);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to load cost ledger",
      );
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchLedger();
  }, []);

  return { entries, isLoading, error, refetch: fetchLedger };
};
