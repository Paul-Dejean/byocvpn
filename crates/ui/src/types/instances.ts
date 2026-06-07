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

export interface SpawnStepState extends SpawnStep {
  status: SpawnStepStatus;
  error?: string;
}

export interface SpawnJobState {
  jobId: string;
  instanceId: string;
  steps: SpawnStepState[];
}
