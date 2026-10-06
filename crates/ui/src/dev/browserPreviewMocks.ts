import { Channel, InvokeArgs } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import {
  CloudProviderName,
  ConnectedInstance,
  InstanceInfo,
  ProviderCredentials,
  SpawnInstanceEvent,
  SpawnJobState,
  VpnStatus,
} from "../bindings";
import {
  PREVIEW_CONFIGURED_PROVIDERS,
  PREVIEW_PRICING,
  PREVIEW_REGIONS,
  PREVIEW_SPAWN_STEPS,
  buildPreviewInstance,
  buildPreviewInstances,
  buildPreviewLedger,
} from "./browserPreviewData";

const VPN_STATUS_EVENT = "vpn-status";
const METRICS_INTERVAL_MILLISECONDS = 1000;
const SPAWN_STEP_INTERVAL_MILLISECONDS = 1500;

const DISCONNECTED_STATUS: VpnStatus = {
  connected: false,
  instance: null,
  metrics: null,
  connectedAt: null,
  connectionError: null,
};

let instances = buildPreviewInstances();
const ledger = buildPreviewLedger(instances);
const spawnJobs = new Map<string, SpawnJobState>();
const storeValues = new Map<string, unknown>();
let vpnStatus: VpnStatus = DISCONNECTED_STATUS;
let metricsTimer: number | null = null;

export function installBrowserPreviewMocks() {
  mockWindows("main");
  mockIPC(handlePreviewCommand, { shouldMockEvents: true });
  console.info("[preview] Tauri backend is mocked with sample data.");
}

function handlePreviewCommand(command: string, args?: InvokeArgs): unknown {
  const commandArgs = (args ?? {}) as Record<string, unknown>;

  switch (command) {
    case "get_credentials":
      return buildPreviewCredentials(commandArgs.provider as CloudProviderName);
    case "has_profile":
      return true;
    case "list_instances":
      return instances;
    case "list_active_spawn_jobs":
      return Array.from(spawnJobs.values());
    case "dismiss_spawn_job":
      spawnJobs.delete(commandArgs.jobId as string);
      return null;
    case "spawn_instance":
      return simulateSpawn(
        commandArgs.provider as CloudProviderName,
        commandArgs.region as string,
        commandArgs.onEvent as Channel<SpawnInstanceEvent>,
      );
    case "terminate_instance":
      instances = instances.filter((instance) => instance.id !== commandArgs.instanceId);
      return `Instance ${commandArgs.instanceId} terminated successfully.`;
    case "connect":
      return connectPreviewVpn(commandArgs);
    case "disconnect":
      return disconnectPreviewVpn();
    case "get_vpn_status":
      return vpnStatus;
    case "get_regions":
      return PREVIEW_REGIONS;
    case "get_instance_pricing":
      return PREVIEW_PRICING;
    case "get_ledger":
      return ledger;
    case "get_notification_settings":
      return {
        notificationEnabled: true,
        notificationThresholdMinutes: 120,
        notificationUnit: "hours",
      };
    case "get_auto_terminate_settings":
      return {
        autoTerminateEnabled: false,
        autoTerminateThresholdMinutes: 30,
        autoTerminateUnit: "minutes",
      };
    case "get_vpn_settings":
      return { sessionKillswitch: true };
    case "verify_permissions":
      return [
        { permission: "ec2:RunInstances", granted: true },
        { permission: "ec2:TerminateInstances", granted: true },
      ];
    case "plugin:store|load":
      return 1;
    case "plugin:store|get":
      return readStoreValue(commandArgs.key as string);
    case "plugin:store|set":
      storeValues.set(commandArgs.key as string, commandArgs.value);
      return null;
    case "plugin:notification|is_permission_granted":
      return true;
    default:
      return null;
  }
}

function buildPreviewCredentials(provider: CloudProviderName): ProviderCredentials | null {
  if (!PREVIEW_CONFIGURED_PROVIDERS.includes(provider)) {
    return null;
  }
  if (provider === "AWS") {
    return {
      provider: "AWS",
      accessKeyId: "AKIAPREVIEWEXAMPLE",
      secretAccessKey: "preview-secret-access-key",
    };
  }
  return {
    provider: "ORACLE",
    tenancyOcid: "ocid1.tenancy.oc1..preview",
    userOcid: "ocid1.user.oc1..preview",
    fingerprint: "12:34:56:78:90:ab:cd:ef",
    privateKeyPem: "-----BEGIN PRIVATE KEY-----",
    region: "us-ashburn-1",
  };
}

function readStoreValue(key: string): [unknown, boolean] {
  return storeValues.has(key) ? [storeValues.get(key), true] : [null, false];
}

function connectPreviewVpn(commandArgs: Record<string, unknown>): string {
  const connectedInstance: ConnectedInstance = {
    instanceId: commandArgs.instanceId as string,
    region: commandArgs.region as string,
    provider: commandArgs.provider as CloudProviderName,
    publicIpV4: (commandArgs.publicIpV4 as string | null) ?? null,
    publicIpV6: (commandArgs.publicIpV6 as string | null) ?? null,
  };
  vpnStatus = {
    connected: true,
    instance: connectedInstance,
    metrics: {
      bytesSent: 0,
      bytesReceived: 0,
      packetsSent: 0,
      packetsReceived: 0,
      uploadRate: 0,
      downloadRate: 0,
    },
    connectedAt: Math.floor(Date.now() / 1000),
    connectionError: null,
  };
  emit(VPN_STATUS_EVENT, vpnStatus);
  startMetricsTimer();
  return `Connected to instance ${connectedInstance.instanceId} successfully.`;
}

function disconnectPreviewVpn(): string {
  if (metricsTimer !== null) {
    window.clearInterval(metricsTimer);
    metricsTimer = null;
  }
  vpnStatus = DISCONNECTED_STATUS;
  emit(VPN_STATUS_EVENT, vpnStatus);
  return "Disconnected successfully.";
}

function startMetricsTimer() {
  if (metricsTimer !== null) {
    window.clearInterval(metricsTimer);
  }
  metricsTimer = window.setInterval(() => {
    if (!vpnStatus.metrics) {
      return;
    }
    const downloadRate = 180_000 + Math.round(Math.random() * 400_000);
    const uploadRate = 20_000 + Math.round(Math.random() * 60_000);
    vpnStatus = {
      ...vpnStatus,
      metrics: {
        ...vpnStatus.metrics,
        bytesReceived: vpnStatus.metrics.bytesReceived + downloadRate,
        bytesSent: vpnStatus.metrics.bytesSent + uploadRate,
        downloadRate,
        uploadRate,
      },
    };
    emit(VPN_STATUS_EVENT, vpnStatus);
  }, METRICS_INTERVAL_MILLISECONDS);
}

function simulateSpawn(
  provider: CloudProviderName,
  region: string,
  channel: Channel<SpawnInstanceEvent>,
): null {
  const job: SpawnJobState = {
    jobId: `${provider}-${Date.now()}`,
    region,
    provider,
    instanceId: null,
    status: "RUNNING",
    error: null,
    steps: PREVIEW_SPAWN_STEPS.map((step) => ({ ...step, status: "PENDING", error: null })),
  };
  spawnJobs.set(job.jobId, job);
  const sendEvent = channel.onmessage;
  sendEvent({ kind: "STARTED", job });

  let completedStepCount = 0;
  const timer = window.setInterval(() => {
    const currentJob = spawnJobs.get(job.jobId);
    if (!currentJob) {
      window.clearInterval(timer);
      return;
    }
    const step = currentJob.steps[completedStepCount];
    step.status = "COMPLETED";
    sendEvent({ kind: "PROGRESS", stepId: step.id, status: "COMPLETED", error: null });
    completedStepCount += 1;

    if (completedStepCount < currentJob.steps.length) {
      currentJob.steps[completedStepCount].status = "RUNNING";
      sendEvent({
        kind: "PROGRESS",
        stepId: currentJob.steps[completedStepCount].id,
        status: "RUNNING",
        error: null,
      });
      return;
    }

    window.clearInterval(timer);
    const instance: InstanceInfo = buildPreviewInstance(provider, region, job.jobId);
    instances = [...instances, instance];
    spawnJobs.delete(job.jobId);
    sendEvent({ kind: "COMPLETE", instance });
  }, SPAWN_STEP_INTERVAL_MILLISECONDS);

  return null;
}
