import { useEffect } from "react";
import { RegionsProvider, InstancesProvider } from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";

interface VpnPageProps {
  onNavigateToAddAccount: () => void;
}

export function VpnPage({ onNavigateToAddAccount }: VpnPageProps) {
  return (
    <RegionsProvider>
      <InstancesProvider>
        <VpnPageContent onNavigateToAddAccount={onNavigateToAddAccount} />
      </InstancesProvider>
    </RegionsProvider>
  );
}

function VpnPageContent({ onNavigateToAddAccount }: VpnPageProps) {
  const { vpnStatus, checkVpnStatus } = useVpnConnectionContext();

  useEffect(() => {
    checkVpnStatus();
  }, []);

  if (vpnStatus.connected && vpnStatus.instance) {
    return <ConnectedView connectedInstance={vpnStatus.instance} />;
  }

  return (
    <ServerManagementView onNavigateToAddAccount={onNavigateToAddAccount} />
  );
}
