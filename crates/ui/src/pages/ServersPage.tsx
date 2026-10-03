import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { CloudProviderName, Instance } from "../types";
import { useInstances } from "../hooks/useInstances";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";
import { configuredProvidersQueryOptions } from "../queries/configuredProviders";
import { ServerCard } from "../components/servers/ServerCard";
import { EmptyServers } from "../components/servers/EmptyServers";
import { SpawnJobCard } from "../components/jobs/SpawnJobCard";
import { ProviderSelector } from "../components/providers/ProviderSelector";
import { RegionSelector } from "../components/regions/RegionSelector";
import { InfoBanner } from "../components/common/InfoBanner";
import { UnprotectedPanel } from "../components/vpn/UnprotectedPanel";
import { ProtectedPanel } from "../components/vpn/ProtectedPanel";
import { Alert } from "../components/primitives/Alert";
import { Button } from "../components/primitives/Button";
import { Spinner } from "../components/primitives/Spinner";

enum DeployStep {
  IDLE = "IDLE",
  SELECTING_PROVIDER = "SELECTING_PROVIDER",
  SELECTING_REGION = "SELECTING_REGION",
}

export function ServersPage() {
  const [deployStep, setDeployStep] = useState<DeployStep>(DeployStep.IDLE);
  const [deployProvider, setDeployProvider] = useState<CloudProviderName>(
    CloudProviderName.Aws,
  );
  const [isBannerDismissed, setIsBannerDismissed] = useState(false);

  const queryClient = useQueryClient();
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

  async function handleAddServer() {
    const configuredProviders = await queryClient.ensureQueryData(
      configuredProvidersQueryOptions,
    );
    if (configuredProviders.length === 1) {
      setDeployProvider(configuredProviders[0]);
      setDeployStep(DeployStep.SELECTING_REGION);
    } else {
      setDeployStep(DeployStep.SELECTING_PROVIDER);
    }
  }

  if (deployStep === DeployStep.SELECTING_PROVIDER) {
    return (
      <ProviderSelector
        onSelectProvider={(provider) => {
          setDeployProvider(provider);
          setDeployStep(DeployStep.SELECTING_REGION);
        }}
        onClose={() => setDeployStep(DeployStep.IDLE)}
      />
    );
  }

  if (deployStep === DeployStep.SELECTING_REGION) {
    return (
      <RegionSelector
        provider={deployProvider}
        onClose={() => setDeployStep(DeployStep.IDLE)}
        onSpawnStarted={() => setDeployStep(DeployStep.IDLE)}
      />
    );
  }

  return (
    <div className="flex h-full">
      <div className="flex-1 min-w-0 flex flex-col py-4 pr-4 gap-3">
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
          <EmptyServers onDeploy={handleAddServer} />
        )}

        {hasServers && (
          <Button
            variant="secondary"
            size="none"
            onClick={handleAddServer}
            icon={<Plus size={16} />}
            className="w-full py-2 text-sm"
          >
            Add server
          </Button>
        )}
      </div>

      <aside className="w-[340px] flex-shrink-0 py-4 pr-4">
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
    </div>
  );
}
