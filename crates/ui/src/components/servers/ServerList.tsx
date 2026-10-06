import { SpawnJobStatus } from "../../types";
import { DeploymentStatus } from "../../contexts/DeploymentsContext";
import { ServersPageState } from "../../hooks/useServersPage";
import { summarizeSpawnJobSteps } from "../../lib/deploymentSteps";
import { DeploymentCard } from "../deploy/DeploymentCard";
import { ServerCard } from "./ServerCard";

interface ServerListProps {
  serversPage: ServersPageState;
}

export function ServerList({ serversPage }: ServerListProps) {
  const {
    activeDeployments,
    untrackedSpawnJobs,
    visibleInstances,
    connectedInstance,
    isConnecting,
    terminatingInstanceId,
    onConnect,
    onTerminate,
    onDismissDeployment,
    onDismissSpawnJob,
  } = serversPage;

  return (
    <>
      {activeDeployments.map((deployment) => (
        <DeploymentCard
          key={deployment.id}
          provider={deployment.provider}
          region={deployment.region.name}
          steps={deployment.steps}
          hasFailed={deployment.status === DeploymentStatus.FAILED}
          error={deployment.error}
          onDismiss={() => onDismissDeployment(deployment.id, deployment.jobId)}
        />
      ))}
      {untrackedSpawnJobs.map((spawnJob) => (
        <DeploymentCard
          key={spawnJob.jobId}
          provider={spawnJob.provider}
          region={spawnJob.region}
          steps={summarizeSpawnJobSteps(spawnJob)}
          hasFailed={spawnJob.status === SpawnJobStatus.Failed}
          error={spawnJob.error}
          onDismiss={() => onDismissSpawnJob(spawnJob.jobId)}
        />
      ))}
      {visibleInstances.map((instance) => (
        <ServerCard
          key={instance.id}
          instance={instance}
          isConnected={connectedInstance?.instanceId === instance.id}
          isConnecting={isConnecting}
          isTerminating={terminatingInstanceId === instance.id}
          onConnect={onConnect}
          onTerminate={onTerminate}
        />
      ))}
    </>
  );
}
