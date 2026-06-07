import { CloudProviderName } from "./providers";

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
  region: string;
  provider: CloudProviderName;
  steps: SpawnStepState[];
}
