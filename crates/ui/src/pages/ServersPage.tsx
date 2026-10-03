import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { Instance } from "../types";
import { useInstances } from "../hooks/useInstances";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";
import { ServerCard } from "../components/servers/ServerCard";
import { EmptyServers } from "../components/servers/EmptyServers";
import { SpawnJobCard } from "../components/jobs/SpawnJobCard";
import { DeployServerModal } from "../components/deploy/DeployServerModal";
import { InfoBanner } from "../components/common/InfoBanner";
import { UnprotectedPanel } from "../components/vpn/UnprotectedPanel";
import { ProtectedPanel } from "../components/vpn/ProtectedPanel";
import { Alert } from "../components/primitives/Alert";
import { Button } from "../components/primitives/Button";
import { Spinner } from "../components/primitives/Spinner";

export function ServersPage() {
  const [isDeployModalOpen, setIsDeployModalOpen] = useState(false);
  const [isBannerDismissed, setIsBannerDismissed] = useState(false);

  const { data: configuredProviders = [] } = useQuery(
    configuredProvidersQueryOptions,
  );
  const {
    instances,
    spawnJobs,
    pendingSpawnJobs,
    isLoading,
    terminatingInstanceId,
    terminateInstance,
    dismissSpawnJob,
    refetchInstances,
  } = useInstances();

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

  const connectedInstance = vpnStatus.connected ? vpnStatus.instance : null;
  const hasServers = instances.length > 0 || pendingSpawnJobs.length > 0;
  const showBanner =
    hasServers && connectedInstance === null && !isBannerDismissed;

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
    <div className="flex h-full min-h-0">
      <div className="flex-1 min-w-0 min-h-0 flex flex-col py-4 pr-4 gap-3">
        <header className="flex flex-col gap-1">
          <h1 className="text-base font-medium text-primary">
            Active servers{" "}
            <span className="text-gray-300 font-normal">[{instances.length}]</span>
          </h1>
          <p className="text-xs text-gray-300">
            Terminate unused servers to stop charges.
          </p>
        </header>

        {showBanner && (
          <InfoBanner onDismiss={() => setIsBannerDismissed(true)}>
            Disconnecting VPN doesn't stop your server. Terminate it when done
            to stop charges.
          </InfoBanner>
        )}

        {vpnError && <Alert variant="error">{vpnError}</Alert>}

        {isLoading && !hasServers ? (
          <div className="flex-1 flex items-center justify-center">
            <Spinner size="w-6 h-6" color="border-gray-400" />
          </div>
        ) : hasServers ? (
          <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-3">
            {pendingSpawnJobs.map((spawnJob) => (
              <SpawnJobCard
                key={spawnJob.jobId}
                spawnJob={spawnJob}
                onDismiss={dismissSpawnJob}
              />
            ))}
            {instances.map((instance) => (
              <ServerCard
                key={instance.id}
                instance={instance}
                isConnected={connectedInstance?.instanceId === instance.id}
                isConnecting={isConnecting}
                isTerminating={terminatingInstanceId === instance.id}
                spawnJob={spawnJobs.find(
                  (spawnJob) => spawnJob.jobId === instance.spawnId,
                )}
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
            size="none"
            onClick={openDeployModal}
            icon={<Plus size={16} />}
            className="w-full py-2 text-sm"
          >
            Add server
          </Button>
        )}
      </div>

      <aside className="w-[340px] flex-shrink-0 min-h-0 py-4 pr-4">
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
