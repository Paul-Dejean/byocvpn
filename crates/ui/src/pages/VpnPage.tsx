import { useEffect } from "react";
import { useInstancesContext } from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { useVpnConnectionContext } from "../contexts/VpnConnectionContext";

export function VpnPage() {
  const { vpnStatus, checkVpnStatus } = useVpnConnectionContext();
  const { refetchInstances } = useInstancesContext();

  useEffect(() => {
    checkVpnStatus();
    refetchInstances();
  }, []);

  if (vpnStatus.connected && vpnStatus.instance) {
    return <ConnectedView connectedInstance={vpnStatus.instance} />;
  }

  return <ServerManagementView />;
}
