import { CloudProviderName } from "./providers";

export enum JobStepStatus {
  Pending = "PENDING",
  Running = "RUNNING",
  Completed = "COMPLETED",
  Failed = "FAILED",
}

export interface JobStep {
  id: string;
  label: string;
}

export interface JobStepState extends JobStep {
  status: JobStepStatus;
  error?: string;
}

export interface SpawnJobState {
  jobId: string;
  instanceId: string;
  region: string;
  provider: CloudProviderName;
  steps: JobStepState[];
}
