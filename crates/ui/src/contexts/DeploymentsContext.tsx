import { createContext, ReactNode, useContext, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { SpawnInstanceEvent } from "../bindings";
import {
  CloudProviderName,
  Instance,
  JobStepState,
  JobStepStatus,
  Region,
} from "../types";
import { useInstances } from "../hooks/useInstances";
import { enableRegion, isRegionEnabled } from "../lib/enableRegion";
import { buildDeploymentSteps } from "../lib/deploymentSteps";

export enum DeploymentStatus {
  RUNNING = "RUNNING",
  COMPLETE = "COMPLETE",
  FAILED = "FAILED",
}

export interface Deployment {
  id: string;
  provider: CloudProviderName;
  region: Region;
  status: DeploymentStatus;
  steps: JobStepState[];
  error: string | null;
  jobId: string | null;
  instance: Instance | null;
}

interface DeploymentsContextValue {
  deployments: Deployment[];
  startDeployment: (provider: CloudProviderName, region: Region) => string;
  dismissDeployment: (deploymentId: string) => void;
}

const DeploymentsContext = createContext<DeploymentsContextValue | null>(null);

interface DeploymentsProviderProps {
  children: ReactNode;
}

export function DeploymentsProvider({ children }: DeploymentsProviderProps) {
  const queryClient = useQueryClient();
  const { startSpawnJob } = useInstances();
  const [deploymentsById, setDeploymentsById] = useState<
    Record<string, Deployment>
  >({});

  function updateDeployment(
    deploymentId: string,
    update: (deployment: Deployment) => Deployment,
  ) {
    setDeploymentsById((previous) => {
      const existing = previous[deploymentId];
      if (!existing) {
        return previous;
      }
      return { ...previous, [deploymentId]: update(existing) };
    });
  }

  function setSteps(
    deploymentId: string,
    deployingStatus: JobStepStatus,
    installingStatus: JobStepStatus,
  ) {
    updateDeployment(deploymentId, (deployment) => ({
      ...deployment,
      steps: buildDeploymentSteps(deployingStatus, installingStatus),
    }));
  }

  function failDeployment(deploymentId: string, error: unknown) {
    updateDeployment(deploymentId, (deployment) => ({
      ...deployment,
      status: DeploymentStatus.FAILED,
      error: error instanceof Error ? error.message : String(error),
      steps: deployment.steps.map((step) =>
        step.status === JobStepStatus.Running
          ? { ...step, status: JobStepStatus.Failed }
          : step,
      ),
    }));
  }

  function applySpawnEvent(deploymentId: string, event: SpawnInstanceEvent) {
    switch (event.kind) {
      case "STARTED":
        updateDeployment(deploymentId, (deployment) => ({
          ...deployment,
          jobId: event.job.jobId,
        }));
        return;
      case "INSTANCE_LAUNCHED":
        setSteps(deploymentId, JobStepStatus.Completed, JobStepStatus.Running);
        return;
      case "COMPLETE":
        updateDeployment(deploymentId, (deployment) => ({
          ...deployment,
          status: DeploymentStatus.COMPLETE,
          instance: event.instance,
          steps: buildDeploymentSteps(JobStepStatus.Completed, JobStepStatus.Completed),
        }));
        return;
      case "FAILED":
        failDeployment(deploymentId, event.error);
        return;
      case "PROGRESS":
        return;
    }
  }

  async function runDeployment(
    deploymentId: string,
    provider: CloudProviderName,
    region: Region,
  ) {
    try {
      const needsEnabling = !(await isRegionEnabled(queryClient, provider, region.name));
      if (needsEnabling) {
        await enableRegion(queryClient, provider, region, () => {});
      }
    } catch (enableError) {
      failDeployment(deploymentId, enableError);
      return;
    }

    const spawnJob = await startSpawnJob(region.name, provider, (event) =>
      applySpawnEvent(deploymentId, event),
    );
    if (!spawnJob) {
      failDeployment(deploymentId, "Failed to start the deployment.");
    }
  }

  function startDeployment(provider: CloudProviderName, region: Region): string {
    const deploymentId = `${provider}-${region.name}-${Date.now()}`;
    setDeploymentsById((previous) => ({
      ...previous,
      [deploymentId]: {
        id: deploymentId,
        provider,
        region,
        status: DeploymentStatus.RUNNING,
        steps: buildDeploymentSteps(JobStepStatus.Running, JobStepStatus.Pending),
        error: null,
        jobId: null,
        instance: null,
      },
    }));
    runDeployment(deploymentId, provider, region);
    return deploymentId;
  }

  function dismissDeployment(deploymentId: string) {
    setDeploymentsById((previous) => {
      const next = { ...previous };
      delete next[deploymentId];
      return next;
    });
  }

  return (
    <DeploymentsContext.Provider
      value={{
        deployments: Object.values(deploymentsById),
        startDeployment,
        dismissDeployment,
      }}
    >
      {children}
    </DeploymentsContext.Provider>
  );
}

export function useDeployments(): DeploymentsContextValue {
  const context = useContext(DeploymentsContext);
  if (!context) {
    throw new Error("useDeployments must be used within DeploymentsProvider");
  }
  return context;
}
