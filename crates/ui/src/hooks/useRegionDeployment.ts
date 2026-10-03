import { useState } from "react";
import { SpawnInstanceEvent } from "../bindings";
import {
  CloudProviderName,
  Instance,
  JobStepState,
  JobStepStatus,
  Region,
} from "../types";
import { useInstances } from "./useInstances";
import { useProviderRegions } from "./useProviderRegions";

export enum DeploymentStatus {
  IDLE = "IDLE",
  RUNNING = "RUNNING",
  COMPLETE = "COMPLETE",
  FAILED = "FAILED",
}

const ENABLE_REGION_STEP_ID = "enable-region";

export function useRegionDeployment(provider: CloudProviderName) {
  const [status, setStatus] = useState<DeploymentStatus>(DeploymentStatus.IDLE);
  const [region, setRegion] = useState<Region | null>(null);
  const [steps, setSteps] = useState<JobStepState[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [instance, setInstance] = useState<Instance | null>(null);

  const { enabledRegions, enableRegion } = useProviderRegions(provider);
  const { startSpawnJob } = useInstances();

  function updateStep(stepId: string, stepStatus: JobStepStatus, stepError: string | null = null) {
    setSteps((previous) =>
      previous.map((step) =>
        step.id === stepId ? { ...step, status: stepStatus, error: stepError } : step,
      ),
    );
  }

  function applySpawnEvent(event: SpawnInstanceEvent) {
    switch (event.kind) {
      case "STARTED":
        setSteps((previous) => [
          ...previous.filter((step) => step.id === ENABLE_REGION_STEP_ID),
          ...event.job.steps,
        ]);
        return;
      case "PROGRESS":
        updateStep(event.stepId, event.status, event.error);
        return;
      case "COMPLETE":
        setInstance(event.instance);
        setStatus(DeploymentStatus.COMPLETE);
        return;
      case "FAILED":
        setError(event.error);
        setStatus(DeploymentStatus.FAILED);
        return;
      case "INSTANCE_LAUNCHED":
        return;
    }
  }

  async function deploy(targetRegion: Region): Promise<void> {
    setRegion(targetRegion);
    setError(null);
    setInstance(null);
    setStatus(DeploymentStatus.RUNNING);

    const needsEnabling = !enabledRegions.has(targetRegion.name);
    setSteps(
      needsEnabling
        ? [
            {
              id: ENABLE_REGION_STEP_ID,
              label: `Enabling ${targetRegion.country}`,
              status: JobStepStatus.Pending,
              error: null,
            },
          ]
        : [],
    );

    if (needsEnabling) {
      try {
        await enableRegion(targetRegion, (stepStatus) =>
          updateStep(ENABLE_REGION_STEP_ID, stepStatus),
        );
      } catch (enableError) {
        setError(
          enableError instanceof Error ? enableError.message : String(enableError),
        );
        setStatus(DeploymentStatus.FAILED);
        return;
      }
    }

    const spawnJob = await startSpawnJob(
      targetRegion.name,
      provider,
      applySpawnEvent,
    );
    if (!spawnJob) {
      setError("Failed to start the deployment.");
      setStatus(DeploymentStatus.FAILED);
    }
  }

  function reset() {
    setStatus(DeploymentStatus.IDLE);
    setRegion(null);
    setSteps([]);
    setError(null);
    setInstance(null);
  }

  return { status, region, steps, error, instance, deploy, reset };
}
