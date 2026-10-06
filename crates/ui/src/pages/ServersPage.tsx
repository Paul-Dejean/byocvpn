import { Plus } from "lucide-react";
import { useServersPage } from "../hooks/useServersPage";
import { ServerList } from "../components/servers/ServerList";
import { EmptyServers } from "../components/servers/EmptyServers";
import { DeployServerModal } from "../components/deploy/DeployServerModal";
import { UnprotectedPanel } from "../components/vpn/UnprotectedPanel";
import { ProtectedPanel } from "../components/vpn/ProtectedPanel";
import { Banner } from "../components/primitives/Banner";
import { Button } from "../components/primitives/Button";
import { Spinner } from "../components/primitives/Spinner";

const SERVERS_PANEL_WIDTH = 464;
const STATUS_PANEL_WIDTH = 358;

export function ServersPage() {
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
    <div className="flex h-full min-h-0 gap-3">
      <section
        className="flex-shrink-0 min-h-0 flex flex-col gap-3 rounded-lg bg-bg-bolder p-3"
        style={{ width: SERVERS_PANEL_WIDTH }}
      >
        {hasServers && (
          <header className="flex flex-col gap-1">
            <h1 className="text-body-sm font-medium text-fg-lighter">
              Active servers{" "}
              <span className="text-fg-medium font-normal">[{instanceCount}]</span>
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
            <ServerList serversPage={serversPage} />
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
            onDisconnect={onDisconnect}
          />
        ) : (
          <UnprotectedPanel hasServers={hasServers} />
        )}
      </aside>

      <DeployServerModal
        isOpen={isDeployModalOpen}
        configuredProviders={configuredProviders}
        onClose={closeDeployModal}
      />
    </div>
  );
}
