import type {
  JobStep as JobStepBinding,
  JobStepStatus as JobStepStatusBinding,
  SpawnJobState,
  SpawnJobStatus as SpawnJobStatusBinding,
  SpawnJobStep,
} from "../bindings";

export const JobStepStatus = {
  Pending: "PENDING",
  Running: "RUNNING",
  Completed: "COMPLETED",
  Failed: "FAILED",
} as const satisfies Record<string, JobStepStatusBinding>;

export type JobStepStatus = JobStepStatusBinding;

export type JobStep = JobStepBinding;

export type JobStepState = SpawnJobStep;

export const SpawnJobStatus = {
  Running: "RUNNING",
  Failed: "FAILED",
} as const satisfies Record<string, SpawnJobStatusBinding>;

export type SpawnJobStatus = SpawnJobStatusBinding;

export type SpawnJob = SpawnJobState;
