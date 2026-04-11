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


export const useLedger = () => {
  const [entries, setEntries] = useState<LedgerEntryWithCost[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchLedger = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const rawEntries = await invoke<LedgerEntry[]>("get_ledger");

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

            pricingCache.set(key, {
              hourlyRate: 0,
              ipHourlyRate: 0,
              egressRatePerGb: 0,
              storageGb: 0,
              storageRatePerGbMonth: 0,
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
        const bytesSentGb = entry.bytesSent / 1024 ** 3;
        const computeCost = uptimeHours * pricing.hourlyRate;
        const ipCost = uptimeHours * pricing.ipHourlyRate;
        const egressCost = bytesSentGb * pricing.egressRatePerGb;
        const storageCost = pricing.storageGb * pricing.storageRatePerGbMonth / 730 * uptimeHours;
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
