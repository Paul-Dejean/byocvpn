export type InstanceState =
  | "running"
  | "creating"
  | "stopping"
  | "stopped"
  | "deleting"
  | "deleted"
  | "unknown";

export interface Instance {
  id: string;
  name: string;
  state: InstanceState | string;
  publicIpV4: string;
  publicIpV6: string;
  region: string;
  provider: string;
}

export type ServerStatus =
  | "idle"
  | "spawning"
  | "running"
  | "error"
  | "terminating"
  | "connecting"
  | "connected";

export type SpawnStepStatus = "pending" | "running" | "completed" | "failed";

export interface SpawnStep {
  id: string;
  label: string;
}

export interface SpawnJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: string;
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

export interface SpawnCompleteEvent {
  jobId: string;
  instance: Instance;
}

export interface SpawnJobState {
  jobId: string;
  steps: SpawnStepState[];
}

export interface ProvisionAccountJob {
  jobId: string;
  steps: SpawnStep[];
  provider: string;
}

export interface ProvisionAccountProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
  error?: string;
}

export interface ProvisionAccountCompleteEvent {
  jobId: string;
  provider: string;
}

export interface EnableRegionJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: string;
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
  provider: string;
}

export interface ProvisionJobState {
  jobId: string;
  provider: string;
  steps: SpawnStepState[];
}
