import { useEffect } from "react";
import { useInstancesContext } from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";

interface VpnPageProps {
  onNavigateToAddAccount: () => void;
}

export function VpnPage({ onNavigateToAddAccount }: VpnPageProps) {
  const { vpnStatus, checkVpnStatus } = useVpnConnectionContext();
  const { refetch } = useInstancesContext();

  useEffect(() => {
    checkVpnStatus();
    refetch();
  }, []);

  if (vpnStatus.connected && vpnStatus.instance) {
    return <ConnectedView connectedInstance={vpnStatus.instance} />;
  }

  return (
    <ServerManagementView onNavigateToAddAccount={onNavigateToAddAccount} />
  );
}
