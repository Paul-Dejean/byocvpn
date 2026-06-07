import { CloudProviderName } from "./providers";

export enum InstanceState {
  Spawning = "SPAWNING",
  Installing = "INSTALLING",
  Error = "ERROR",
  Running = "RUNNING",
  Creating = "CREATING",
  Stopping = "STOPPING",
  Stopped = "STOPPED",
  Deleting = "DELETING",
  Deleted = "DELETED",
  Unknown = "UNKNOWN",
}

export interface Instance {
  id: string;
  name: string;
  state: InstanceState;
  errorReason?: string;
  publicIpV4: string;
  publicIpV6: string;
  region: string;
  provider: CloudProviderName;
}

export type ServerStatus =
  | "idle"
  | "spawning"
  | "running"
  | "error"
  | "terminating"
  | "connecting"
  | "connected";

export enum SpawnStepStatus {
  Pending = "PENDING",
  Running = "RUNNING",
  Completed = "COMPLETED",
  Failed = "FAILED",
}

export interface SpawnStep {
  id: string;
  label: string;
}

export interface SpawnJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: CloudProviderName;
}

export interface SpawnStepState extends SpawnStep {
  status: SpawnStepStatus;
  error?: string;
}

export interface SpawnProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
  error?: string;
}

export interface SpawnInstanceLaunchedEvent {
  jobId: string;
  instance: Instance;
}

export interface SpawnCompleteEvent {
  jobId: string;
  instance: Instance;
}

export interface ActiveSpawnJob extends SpawnJob {
  instanceId: string | null;
  stepStatuses: Record<string, SpawnStepStatus>;
}

export interface SpawnJobState {
  jobId: string;
  instanceId: string;
  steps: SpawnStepState[];
}

export interface ProvisionAccountJob {
  jobId: string;
  steps: SpawnStep[];
  provider: CloudProviderName;
}

export interface ProvisionAccountProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
  error?: string;
}

export interface ProvisionAccountCompleteEvent {
  jobId: string;
  provider: CloudProviderName;
}

export interface EnableRegionJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: CloudProviderName;
}

export interface EnableRegionProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
  error?: string;
}

export interface EnableRegionCompleteEvent {
  jobId: string;
  region: string;
  provider: CloudProviderName;
}

export interface ProvisionJobState {
  jobId: string;
  provider: CloudProviderName;
  steps: SpawnStepState[];
}
