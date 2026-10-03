import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { Instance, SpawnJobStatus } from "../types";
import { useInstances } from "../hooks/useInstances";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";
import {
  DeploymentStatus,
  useDeployments,
} from "../contexts/DeploymentsContext";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";
import { summarizeSpawnJobSteps } from "../lib/deploymentSteps";
import { ServerCard } from "../components/servers/ServerCard";
import { EmptyServers } from "../components/servers/EmptyServers";
import { DeploymentCard } from "../components/deploy/DeploymentCard";
import { DeployServerModal } from "../components/deploy/DeployServerModal";
import { UnprotectedPanel } from "../components/vpn/UnprotectedPanel";
import { ProtectedPanel } from "../components/vpn/ProtectedPanel";
import { Banner } from "../components/primitives/Banner";
import { Button } from "../components/primitives/Button";
import { Spinner } from "../components/primitives/Spinner";

const SERVERS_PANEL_WIDTH = 464;
const STATUS_PANEL_WIDTH = 358;

export function ServersPage() {
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

  async function handleConnect(instance: Instance) {
    clearError();
    await connectToVpn(instance);
  }

  async function handleTerminate(instance: Instance) {
    clearError();
    await terminateInstance(instance.id, instance.region, instance.provider);
  }

  function openDeployModal() {
    setIsDeployModalOpen(true);
  }

  return (
    <div className="flex h-full min-h-0 gap-3">
      <section
        className="flex-shrink-0 min-h-0 flex flex-col gap-3 rounded-lg bg-bg-bolder p-3"
        style={{ width: SERVERS_PANEL_WIDTH }}
      >
        {hasServers && (
          <header className="flex flex-col gap-1">
            <h1 className="text-body-sm font-medium text-fg-lighter">
              Active servers{" "}
              <span className="text-fg-medium font-normal">[{instances.length}]</span>
            </h1>
            <p className="text-caption text-fg-medium">
              Terminate unused servers to stop charges.
            </p>
          </header>
        )}

        {vpnError && <Banner variant="danger">{vpnError}</Banner>}

        {isLoading && !hasServers ? (
          <div className="flex-1 flex items-center justify-center">
            <Spinner size="w-6 h-6" color="border-bd-strong" />
          </div>
        ) : hasServers ? (
          <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-3">
            {activeDeployments.map((deployment) => (
              <DeploymentCard
                key={deployment.id}
                provider={deployment.provider}
                region={deployment.region.name}
                steps={deployment.steps}
                hasFailed={deployment.status === DeploymentStatus.FAILED}
                error={deployment.error}
                onDismiss={() => {
                  if (deployment.jobId) {
                    dismissSpawnJob(deployment.jobId);
                  }
                  dismissDeployment(deployment.id);
                }}
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
                onDismiss={() => dismissSpawnJob(spawnJob.jobId)}
              />
            ))}
            {visibleInstances.map((instance) => (
              <ServerCard
                key={instance.id}
                instance={instance}
                isConnected={connectedInstance?.instanceId === instance.id}
                isConnecting={isConnecting}
                isTerminating={terminatingInstanceId === instance.id}
                onConnect={handleConnect}
                onTerminate={handleTerminate}
              />
            ))}
          </div>
        ) : (
          <EmptyServers onDeploy={openDeployModal} />
        )}

        {hasServers && (
          <Button
            variant="secondary"
            size="lg"
            onClick={openDeployModal}
            icon={<Plus size={14} />}
            className="w-full"
          >
            Add server
          </Button>
        )}
      </section>

      <aside
        className="flex-shrink-0 min-h-0 rounded-lg bg-bg-bolder py-3 px-4"
        style={{ width: STATUS_PANEL_WIDTH }}
      >
        {connectedInstance ? (
          <ProtectedPanel
            connectedInstance={connectedInstance}
            metrics={vpnStatus.metrics}
            connectedAt={vpnStatus.connectedAt}
            isDisconnecting={isDisconnecting}
            isDaemonRunning={isDaemonRunning}
            onDisconnect={disconnectFromVpn}
          />
        ) : (
          <UnprotectedPanel hasServers={hasServers} />
        )}
      </aside>

      <DeployServerModal
        isOpen={isDeployModalOpen}
        configuredProviders={configuredProviders}
        onClose={() => setIsDeployModalOpen(false)}
      />
    </div>
  );
}
