import type {
  SpawnJobState,
  SpawnJobStatus as SpawnJobStatusBinding,
  SpawnJobStep,
  SpawnStep,
  SpawnStepStatus,
} from "../bindings";

export const JobStepStatus = {
  Pending: "PENDING",
  Running: "RUNNING",
  Completed: "COMPLETED",
  Failed: "FAILED",
} as const satisfies Record<string, SpawnStepStatus>;

export type JobStepStatus = SpawnStepStatus;

export type JobStep = SpawnStep;

export type JobStepState = SpawnJobStep;

export const SpawnJobStatus = {
  Running: "RUNNING",
  Failed: "FAILED",
} as const satisfies Record<string, SpawnJobStatusBinding>;

export type SpawnJobStatus = SpawnJobStatusBinding;

export type SpawnJob = SpawnJobState;
