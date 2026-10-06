import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Instance } from "../types";
import { useInstances } from "./useInstances";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";
import {
  DeploymentStatus,
  useDeployments,
} from "../contexts/DeploymentsContext";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";

export function useServersPage() {
  const [isDeployModalOpen, setIsDeployModalOpen] = useState(false);

  const { data: configuredProviders = [] } = useQuery(
    configuredProvidersQueryOptions,
  );
  const {
    instances,
    pendingSpawnJobs,
    isLoading,
    terminatingInstanceId,
    terminateInstance,
    dismissSpawnJob,
    refetchInstances,
  } = useInstances();
  const { deployments, dismissDeployment } = useDeployments();

  const {
    vpnStatus,
    checkVpnStatus,
    isConnecting,
    isDisconnecting,
    isDaemonRunning,
    error: vpnError,
    connectToVpn,
    disconnectFromVpn,
    clearError,
  } = useVpnConnectionContext();

  useEffect(() => {
    checkVpnStatus();
    refetchInstances();
  }, []);

  const activeDeployments = deployments.filter(
    (deployment) => deployment.status !== DeploymentStatus.COMPLETE,
  );
  const trackedJobIds = new Set(
    activeDeployments
      .map((deployment) => deployment.jobId)
      .filter((jobId): jobId is string => jobId !== null),
  );
  const untrackedSpawnJobs = pendingSpawnJobs.filter(
    (spawnJob) => !trackedJobIds.has(spawnJob.jobId),
  );
  const visibleInstances = instances.filter(
    (instance) => instance.spawnId === null || !trackedJobIds.has(instance.spawnId),
  );

  const connectedInstance = vpnStatus.connected ? vpnStatus.instance : null;
  const hasServers =
    visibleInstances.length > 0 ||
    untrackedSpawnJobs.length > 0 ||
    activeDeployments.length > 0;

  async function onConnect(instance: Instance) {
    clearError();
    await connectToVpn(instance);
  }

  async function onTerminate(instance: Instance) {
    clearError();
    await terminateInstance(instance.id, instance.region, instance.provider);
  }

  function onDismissDeployment(deploymentId: string, jobId: string | null) {
    if (jobId) {
      dismissSpawnJob(jobId);
    }
    dismissDeployment(deploymentId);
  }

  return {
    configuredProviders,
    isDeployModalOpen,
    openDeployModal: () => setIsDeployModalOpen(true),
    closeDeployModal: () => setIsDeployModalOpen(false),
    instanceCount: instances.length,
    activeDeployments,
    untrackedSpawnJobs,
    visibleInstances,
    connectedInstance,
    hasServers,
    isLoading,
    terminatingInstanceId,
    vpnStatus,
    vpnError,
    isConnecting,
    isDisconnecting,
    isDaemonRunning,
    onConnect,
    onTerminate,
    onDisconnect: disconnectFromVpn,
    onDismissDeployment,
    onDismissSpawnJob: dismissSpawnJob,
  };
}

export type ServersPageState = ReturnType<typeof useServersPage>;
