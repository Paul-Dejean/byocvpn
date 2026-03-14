import { useEffect } from "react";
import { RegionsProvider, InstancesProvider } from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import {
  useVpnConnectionContext,
  VpnConnectionProvider,
} from "../contexts/VpnConnectionContext";

interface VpnPageProps {

  onNavigateToSettings: () => void;
}

export function VpnPage({ onNavigateToSettings }: VpnPageProps) {
  return (
    <RegionsProvider>
      <InstancesProvider>
        <VpnConnectionProvider>
          <VpnPageContent onNavigateToSettings={onNavigateToSettings} />
        </VpnConnectionProvider>
      </InstancesProvider>
    </RegionsProvider>
  );
}

function VpnPageContent({ onNavigateToSettings }: VpnPageProps) {
  const { vpnStatus, checkVpnStatus } = useVpnConnectionContext();

  useEffect(() => {
    checkVpnStatus();
  }, []);

  if (vpnStatus.connected && vpnStatus.instance) {
    return (
      <ConnectedView
        connectedInstance={vpnStatus.instance}
        onNavigateToSettings={onNavigateToSettings}
      />
    );
  }

  return <ServerManagementView onNavigateToSettings={onNavigateToSettings} />;
}
