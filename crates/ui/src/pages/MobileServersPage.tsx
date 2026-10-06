import { Plus } from "lucide-react";
import { useServersPage } from "../hooks/useServersPage";
import { ServerList } from "../components/servers/ServerList";
import { EmptyServers } from "../components/servers/EmptyServers";
import { DeployServerModal } from "../components/deploy/DeployServerModal";
import {
  ProtectedStatusCard,
  UnprotectedStatusCard,
} from "../components/vpn/MobileStatusCards";
import { Banner } from "../components/primitives/Banner";
import { Button } from "../components/primitives/Button";
import { Spinner } from "../components/primitives/Spinner";

export function MobileServersPage() {
  const serversPage = useServersPage();
  const {
    configuredProviders,
    isDeployModalOpen,
    openDeployModal,
    closeDeployModal,
    instanceCount,
    connectedInstance,
    hasServers,
    isLoading,
    vpnStatus,
    vpnError,
    isDisconnecting,
    isDaemonRunning,
    onDisconnect,
  } = serversPage;

  return (
    <div className="h-full min-h-0 overflow-y-auto flex flex-col gap-4">
      {connectedInstance ? (
        <ProtectedStatusCard
          connectedInstance={connectedInstance}
          metrics={vpnStatus.metrics}
          connectedAt={vpnStatus.connectedAt}
          isDisconnecting={isDisconnecting}
          isDaemonRunning={isDaemonRunning}
          onDisconnect={onDisconnect}
        />
      ) : (
        <UnprotectedStatusCard hasServers={hasServers} />
      )}

      {vpnError && <Banner variant="danger">{vpnError}</Banner>}

      {isLoading && !hasServers ? (
        <div className="flex justify-center py-10">
          <Spinner size="w-6 h-6" color="border-bd-strong" />
        </div>
      ) : hasServers ? (
        <section className="flex flex-col gap-3">
          <header className="flex flex-col gap-1">
            <h1 className="text-body-sm font-medium text-fg-lighter">
              Active servers{" "}
              <span className="text-fg-medium font-normal">[{instanceCount}]</span>
            </h1>
            <p className="text-caption text-fg-medium">
              Terminate unused servers to stop charges.
            </p>
          </header>
          <ServerList serversPage={serversPage} />
          <Button
            variant="secondary"
            size="lg"
            onClick={openDeployModal}
            icon={<Plus size={14} />}
            className="w-full"
          >
            Add server
          </Button>
        </section>
      ) : (
        <div className="py-6 flex">
          <EmptyServers onDeploy={openDeployModal} />
        </div>
      )}

      <DeployServerModal
        isOpen={isDeployModalOpen}
        configuredProviders={configuredProviders}
        onClose={closeDeployModal}
      />
    </div>
  );
}
