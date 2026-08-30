import type { LedgerEntry, PricingInfo } from "../bindings";

export type { LedgerEntry, PricingInfo };

export interface LedgerEntryWithCost extends LedgerEntry {
  isPricingUnknown: boolean;
  estimatedCost: number;
  uptimeHours: number;
  computeCost: number;
  ipCost: number;
  egressCost: number;
  storageCost: number;
  storageGb: number;
  storageRatePerGbMonth: number;
}
