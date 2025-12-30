import { useEffect, useState } from "react";
import { useVpnConnection } from "../hooks";
import {
  RegionsProvider,
  InstancesProvider,
  useInstancesContext,
} from "../contexts";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { Instance } from "../types";

/**
 * Props for the VpnPage component
 */
interface VpnPageProps {
  /** Callback to navigate to settings page */
  onNavigateToSettings?: () => void;
}

/**
 * Main VPN page that handles routing between connected and management views
 */
export function VpnPage({ onNavigateToSettings }: VpnPageProps) {
  return (
    <RegionsProvider>
      <InstancesProvider>
        <VpnPageContent onNavigateToSettings={onNavigateToSettings} />
      </InstancesProvider>
    </RegionsProvider>
  );
}

/**
 * Inner component that uses the contexts
 */
function VpnPageContent({ onNavigateToSettings }: VpnPageProps) {
  const { vpnStatus, checkVpnStatus } = useVpnConnection();
  const { instances } = useInstancesContext();

  const [connectedInstance, setConnectedInstance] = useState<Instance | null>(
    null
  );

  useEffect(() => {
    // Check VPN status on mount and when instances change
    checkVpnStatus();
  }, [instances]);

  useEffect(() => {
    // Poll VPN status every 2 seconds to catch connection changes
    const interval = setInterval(() => {
      checkVpnStatus();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (vpnStatus.connected) {
      const region = instances.find(
        (inst) => inst.id === vpnStatus.instance.id
      )?.region;
      setConnectedInstance({ ...vpnStatus.instance, region: region ?? "" });
    } else {
      setConnectedInstance(null);
    }
  }, [vpnStatus, instances]);

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
