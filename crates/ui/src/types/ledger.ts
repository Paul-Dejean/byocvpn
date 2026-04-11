export interface LedgerEntry {
  instanceId: string;
  provider: string;
  region: string;
  instanceType: string;
  launchedAt: string;
  terminatedAt: string | null;
  bytesSent: number;
  bytesReceived: number;
}

export interface PricingInfo {
  hourlyRate: number;
  ipHourlyRate: number;
  egressRatePerGb: number;
  storageGb: number;
  storageRatePerGbMonth: number;
}

export interface LedgerEntryWithCost extends LedgerEntry {
  estimatedCost: number;
  uptimeHours: number;
  computeCost: number;
  ipCost: number;
  egressCost: number;
  storageCost: number;
  storageGb: number;
  storageRatePerGbMonth: number;
}
