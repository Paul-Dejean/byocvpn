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

export enum SpawnJobStatus {
  Running = "RUNNING",
  Failed = "FAILED",
}

export interface SpawnJob {
  jobId: string;
  region: string;
  provider: CloudProviderName;
  instanceId: string | null;
  status: SpawnJobStatus;
  error?: string;
  steps: JobStepState[];
}
