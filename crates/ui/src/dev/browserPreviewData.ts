import {
  CloudProviderName,
  InstanceInfo,
  LedgerEntry,
  PricingInfo,
  Region,
  SpawnStep,
} from "../bindings";

const HOUR_IN_MILLISECONDS = 3_600_000;

export const PREVIEW_CONFIGURED_PROVIDERS: CloudProviderName[] = ["AWS", "ORACLE"];

export const PREVIEW_REGIONS: Region[] = [
  { name: "eu-central-1", country: "Germany" },
  { name: "eu-west-3", country: "France" },
  { name: "us-east-1", country: "United States" },
  { name: "ap-northeast-1", country: "Japan" },
  { name: "sa-east-1", country: "Brazil" },
];

export const PREVIEW_PRICING: PricingInfo = {
  hourlyRate: 0.0042,
  ipHourlyRate: 0.005,
  egressRatePerGb: 0.09,
  storageGb: 8,
  storageRatePerGbMonth: 0.08,
};

export const PREVIEW_SPAWN_STEPS: SpawnStep[] = [
  { id: "network", label: "Create network" },
  { id: "launch", label: "Launch instance" },
  { id: "install", label: "Install WireGuard" },
];

export function buildPreviewInstances(): InstanceInfo[] {
  return [
    {
      id: "i-0f3a9c2e81b74d5a6",
      name: null,
      region: "eu-central-1",
      state: "RUNNING",
      errorReason: null,
      publicIpV4: "18.194.22.10",
      publicIpV6: "2a05:d014:9f2:4c00::1a",
      provider: "AWS",
      instanceType: "t4g.nano",
      launchedAt: buildIsoTimestamp(-2.5 * HOUR_IN_MILLISECONDS),
      spawnId: null,
    },
    {
      id: "ocid1.instance.oc1.iad.anuwcljt3xk7",
      name: null,
      region: "us-ashburn-1",
      state: "RUNNING",
      errorReason: null,
      publicIpV4: "129.213.40.77",
      publicIpV6: "",
      provider: "ORACLE",
      instanceType: "VM.Standard.E2.1.Micro",
      launchedAt: buildIsoTimestamp(-26 * HOUR_IN_MILLISECONDS),
      spawnId: null,
    },
  ];
}

export function buildPreviewLedger(instances: InstanceInfo[]): LedgerEntry[] {
  const runningEntries: LedgerEntry[] = instances.map((instance) => ({
    instanceId: instance.id,
    provider: instance.provider,
    region: instance.region,
    instanceType: instance.instanceType,
    launchedAt: instance.launchedAt ?? buildIsoTimestamp(0),
    terminatedAt: null,
    bytesSent: 412_000_000,
    bytesReceived: 2_380_000_000,
  }));

  return [
    ...runningEntries,
    {
      instanceId: "i-07c1e5d2a9f4b3e80",
      provider: "AWS",
      region: "ap-northeast-1",
      instanceType: "t4g.nano",
      launchedAt: buildIsoTimestamp(-120 * HOUR_IN_MILLISECONDS),
      terminatedAt: buildIsoTimestamp(-117 * HOUR_IN_MILLISECONDS),
      bytesSent: 95_000_000,
      bytesReceived: 640_000_000,
    },
  ];
}

export function buildPreviewInstance(
  provider: CloudProviderName,
  region: string,
  spawnId: string,
): InstanceInfo {
  return {
    id: `i-preview-${Date.now().toString(16)}`,
    name: null,
    region,
    state: "RUNNING",
    errorReason: null,
    publicIpV4: "3.121.58.204",
    publicIpV6: "",
    provider,
    instanceType: "t4g.nano",
    launchedAt: buildIsoTimestamp(0),
    spawnId,
  };
}

function buildIsoTimestamp(offsetMilliseconds: number): string {
  return new Date(Date.now() + offsetMilliseconds).toISOString();
}
