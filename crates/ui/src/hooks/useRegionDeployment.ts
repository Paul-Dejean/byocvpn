import { useState } from "react";
import { CloudProviderName, Region } from "../types";
import {
  Deployment,
  DeploymentStatus,
  useDeployments,
} from "../contexts/DeploymentsContext";

export { DeploymentStatus };

export function useRegionDeployment(provider: CloudProviderName) {
  const { deployments, startDeployment, dismissDeployment } = useDeployments();
  const [deploymentId, setDeploymentId] = useState<string | null>(null);

  const deployment: Deployment | null =
    deployments.find((candidate) => candidate.id === deploymentId) ?? null;

  function deploy(region: Region) {
    setDeploymentId(startDeployment(provider, region));
  }

  function reset() {
    if (deploymentId) {
      dismissDeployment(deploymentId);
    }
    setDeploymentId(null);
  }

  return {
    status: deployment?.status ?? null,
    steps: deployment?.steps ?? [],
    error: deployment?.error ?? null,
    instance: deployment?.instance ?? null,
    deploy,
    reset,
  };
}
