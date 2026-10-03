import { JobStepState, JobStepStatus, SpawnJob, SpawnJobStatus } from "../types";

export const DEPLOYING_STEP_ID = "deploying";
export const INSTALLING_STEP_ID = "installing";

export function buildDeploymentSteps(
  deployingStatus: JobStepStatus,
  installingStatus: JobStepStatus,
): JobStepState[] {
  return [
    { id: DEPLOYING_STEP_ID, label: "Deploying", status: deployingStatus, error: null },
    { id: INSTALLING_STEP_ID, label: "Installing", status: installingStatus, error: null },
  ];
}

export function summarizeSpawnJobSteps(spawnJob: SpawnJob): JobStepState[] {
  const hasLaunched = spawnJob.instanceId !== null;
  if (spawnJob.status === SpawnJobStatus.Failed) {
    return hasLaunched
      ? buildDeploymentSteps(JobStepStatus.Completed, JobStepStatus.Failed)
      : buildDeploymentSteps(JobStepStatus.Failed, JobStepStatus.Pending);
  }
  return hasLaunched
    ? buildDeploymentSteps(JobStepStatus.Completed, JobStepStatus.Running)
    : buildDeploymentSteps(JobStepStatus.Running, JobStepStatus.Pending);
}
