import { useEffect, useState } from "react";
import { RegionsProvider, InstancesProvider } from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { Instance } from "../types";
import {
  useVpnConnectionContext,
  VpnConnectionProvider,
} from "../contexts/VpnConnectionContext";

/**
 * Props for the VpnPage component
 */
interface VpnPageProps {
  /** Callback to navigate to settings page */
  onNavigateToSettings: () => void;
}

/**
 * Main VPN page that handles routing between connected and management views
 */
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

/**
 * Inner component that uses the contexts
 */
function VpnPageContent({ onNavigateToSettings }: VpnPageProps) {
  const { vpnStatus, checkVpnStatus } = useVpnConnectionContext();

  const [connectedInstance, setConnectedInstance] = useState<Instance | null>(
    null
  );

  useEffect(() => {
    checkVpnStatus();
  }, []);

  useEffect(() => {
    console.log("effect", vpnStatus);
    if (vpnStatus.connected) {
      setConnectedInstance(vpnStatus.instance);
    } else {
      setConnectedInstance(null);
    }
  }, [vpnStatus]);

  if (vpnStatus.connected && connectedInstance) {
    return (
      <ConnectedView
        connectedInstance={connectedInstance}
        onNavigateToSettings={onNavigateToSettings}
      />
    );
  }

  return <ServerManagementView onNavigateToSettings={onNavigateToSettings} />;
}
