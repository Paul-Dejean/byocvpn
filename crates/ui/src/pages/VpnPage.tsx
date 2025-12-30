import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { useRegions, useInstances } from "../hooks";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";
import { ExistingInstance, ServerDetails } from "../types";

/**
 * VPN server status states
 */
enum ServerStatus {
  IDLE = "idle",
  CONNECTED = "connected",
}

/**
 * Props for the VpnPage component
 */
interface VpnPageProps {
  /** Callback to navigate to settings page */
  onNavigateToSettings?: () => void;
}

/**
 * VPN status response from backend
 */
interface VpnStatus {
  connected: boolean;
  instance_id?: string;
}

/**
 * Main VPN page that handles routing between connected and management views
 */
export function VpnPage({ onNavigateToSettings }: VpnPageProps) {
  const [serverStatus, setServerStatus] = useState<ServerStatus>(
    ServerStatus.IDLE
  );
  const [checkingStatus, setCheckingStatus] = useState(true);

  const { regions } = useRegions();
  const { existingInstances, selectedInstance, setSelectedInstance } =
    useInstances(regions);

  useEffect(() => {
    checkVpnStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [existingInstances]);

  const checkVpnStatus = async () => {
    try {
      const status = await invoke<VpnStatus>("get_vpn_status");
      console.log("VPN Status on mount:", status);

      if (status.connected) {
        await handleConnectedStatus(status);
      }
    } catch (error) {
      console.error("Failed to check VPN status:", error);
    } finally {
      setCheckingStatus(false);
    }
  };

  const handleConnectedStatus = async (status: VpnStatus) => {
    console.log("Restoring VPN connection state");
    setServerStatus(ServerStatus.CONNECTED);

    await restartMetricsStream();

    if (status.instance_id) {
      restoreSelectedInstance(status.instance_id);
    } else {
      toast.success("VPN connection restored");
    }
  };

  const restartMetricsStream = async () => {
    try {
      console.log("Restarting metrics stream...");
      await invoke("start_metrics_stream");
      console.log("Metrics stream restarted successfully");
    } catch (error) {
      console.error("Failed to restart metrics stream:", error);
    }
  };

  const restoreSelectedInstance = (instanceId: string) => {
    console.log("Looking for instance:", instanceId);

    const instance = existingInstances.find((i) => i.id === instanceId);

    if (instance?.region) {
      console.log("Found matching instance:", instance);
      const serverDetails = createServerDetailsFromInstance(instance);
      setSelectedInstance(serverDetails);
      toast.success(`VPN connected to ${instance.name || instance.id}`);
    } else {
      console.log("Instance not found in list");
      toast.success("VPN connection restored");
    }
  };

  const handleDisconnect = async () => {
    setServerStatus(ServerStatus.IDLE);
  };

  if (checkingStatus) {
    return <LoadingView />;
  }

  if (serverStatus === ServerStatus.CONNECTED) {
    return (
      <ConnectedView
        selectedInstance={selectedInstance}
        onDisconnect={handleDisconnect}
        onNavigateToSettings={onNavigateToSettings}
      />
    );
  }

  return <ServerManagementView onNavigateToSettings={onNavigateToSettings} />;
}

/**
 * Creates ServerDetails object from ExistingInstance
 */
function createServerDetailsFromInstance(
  instance: ExistingInstance
): ServerDetails {
  return {
    instance_id: instance.id,
    public_ip_v4: instance.public_ip_v4 || "",
    public_ip_v6: instance.public_ip_v6 || "",
    region: instance.region || "",
    client_private_key: "",
    server_public_key: "",
  };
}

/**
 * Loading view shown while checking VPN status
 */
function LoadingView() {
  return (
    <div className="flex flex-col items-center justify-center h-screen bg-gray-900 text-white">
      <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mb-4"></div>
      <p className="text-gray-400">Checking VPN status...</p>
    </div>
  );
}
