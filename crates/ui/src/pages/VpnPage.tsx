import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import {
  useRegions,
  useInstances,
  useVpnConnection,
  useSpawnInstance,
  useTerminateInstance,
  useVpnMetrics,
} from "../hooks";
import { AwsRegion, ExistingInstance } from "../types";
import { ConnectedView } from "../components/vpn/ConnectedView";
import { ServerManagementView } from "../components/vpn/ServerManagementView";

interface VpnPageProps {
  onNavigateToSettings?: () => void;
}

export function VpnPage({ onNavigateToSettings }: VpnPageProps) {
  // Hooks for state management
  const {
    regions,
    groupedRegions,
    selectedRegion,
    isLoading: regionsLoading,
    error: regionsError,
    handleRegionSelect,
    clearError: clearRegionsError,
  } = useRegions();

  const {
    existingInstances,
    selectedInstance,
    isLoading: instancesLoading,
    error: instancesError,
    handleInstanceSelect,
    addInstance,
    removeInstance,
    clearSelectedInstance,
    setSelectedInstance,
    clearError: clearInstancesError,
  } = useInstances(regions);

  const {
    isSpawning,
    error: spawnError,
    spawnInstance,
    clearError: clearSpawnError,
  } = useSpawnInstance();

  const {
    isTerminating,
    error: terminateError,
    terminateInstance,
    clearError: clearTerminateError,
  } = useTerminateInstance();

  const {
    serverStatus,
    isConnecting,
    error: vpnError,
    setServerStatus,
    handleConnectToVpn,
    handleDisconnectFromVpn,
    clearError: clearVpnError,
  } = useVpnConnection();

  // VPN Metrics
  const metrics = useVpnMetrics(serverStatus === "connected");

  // Local state removed - now managed in hooks
  const [checkingStatus, setCheckingStatus] = useState(true);

  const handleRegionSelectWrapper = (region: AwsRegion) => {
    handleRegionSelect(region);
    // Clear selected instance when selecting a region (to show spawn controls)
    clearSelectedInstance();
    setServerStatus("idle");
  };

  const handleInstanceSelectWrapper = (instance: ExistingInstance) => {
    handleInstanceSelect(instance);
    setServerStatus("running");
  };

  const handleSpawnServer = async () => {
    if (!selectedRegion) return;

    setServerStatus("spawning");

    try {
      const serverDetails = await spawnInstance(selectedRegion.name);

      // Add the new instance to local state
      const newInstance: ExistingInstance = {
        id: serverDetails.instance_id,
        name: "VPN Server",
        state: "running",
        public_ip_v4: serverDetails.public_ip_v4,
        public_ip_v6: serverDetails.public_ip_v6 || "",
        region: selectedRegion.name,
      };

      addInstance(newInstance);
      setSelectedInstance(serverDetails);
      setServerStatus("running");
    } catch (error) {
      console.error("Failed to spawn server:", error);
      setServerStatus("error");
    }
  };

  const handleTerminateServer = async () => {
    if (!selectedInstance) return;

    setServerStatus("terminating");

    try {
      await terminateInstance(
        selectedInstance.instance_id,
        selectedInstance.region
      );
      removeInstance(selectedInstance.instance_id);
      clearSelectedInstance();
      setServerStatus("idle");
    } catch (error) {
      console.error("Failed to terminate server:", error);
      setServerStatus("error");
    }
  };

  // Check VPN status on mount
  useEffect(() => {
    async function checkStatus() {
      try {
        const status: any = await invoke("get_vpn_status");
        console.log("VPN Status on mount:", status);
        if (status.connected) {
          console.log("Restoring VPN connection state");
          setServerStatus("connected");

          // Restart metrics stream
          try {
            console.log("Restarting metrics stream...");
            await invoke("start_metrics_stream");
            console.log("Metrics stream restarted successfully");
          } catch (metricsErr) {
            console.error("Failed to restart metrics stream:", metricsErr);
          }

          if (status.instance_id) {
            console.log("Looking for instance:", status.instance_id);
            // Find the instance in existingInstances and select it
            const instance = existingInstances.find(
              (i) => i.id === status.instance_id
            );
            if (instance && instance.region) {
              console.log("Found matching instance:", instance);

              // Create ServerDetails object to properly set the selected instance
              const serverDetails = {
                instance_id: instance.id,
                public_ip_v4: instance.public_ip_v4 || "",
                public_ip_v6: instance.public_ip_v6 || "",
                region: instance.region,
                client_private_key: "", // Not needed for display
                server_public_key: "", // Not needed for display
              };

              setSelectedInstance(serverDetails);
              toast.success(`VPN connected to ${instance.name || instance.id}`);
            } else {
              console.log(
                "Instance not found in list, showing generic message"
              );
              toast.success("VPN connection restored");
            }
          } else {
            console.log("No instance_id in status");
            toast.success("VPN connection restored");
          }
        } else {
          console.log("VPN not connected");
        }
      } catch (err) {
        console.error("Failed to check VPN status:", err);
      } finally {
        setCheckingStatus(false);
      }
    }

    checkStatus();
  }, [existingInstances, setServerStatus, setSelectedInstance]);

  const isLoading = regionsLoading || instancesLoading || checkingStatus;

  // Show loading state
  if (checkingStatus) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-gray-900 text-white">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mb-4"></div>
        <p className="text-gray-400">Checking VPN status...</p>
      </div>
    );
  }

  // Show Connected View if VPN is active
  if (serverStatus === "connected") {
    const serverInfo = selectedInstance
      ? {
          instanceId: selectedInstance.instance_id,
          region: selectedInstance.region,
          publicIpv4: selectedInstance.public_ip_v4,
          publicIpv6: selectedInstance.public_ip_v6 || undefined,
        }
      : undefined;

    return (
      <ConnectedView
        metrics={metrics}
        serverInfo={serverInfo}
        onDisconnect={handleDisconnectFromVpn}
        onNavigateToSettings={onNavigateToSettings}
        isDisconnecting={false}
      />
    );
  }

  // Show Server Management View
  return (
    <ServerManagementView
      groupedRegions={groupedRegions}
      selectedRegion={selectedRegion}
      onRegionSelect={handleRegionSelectWrapper}
      regionsError={regionsError}
      existingInstances={existingInstances}
      selectedInstance={selectedInstance}
      onInstanceSelect={handleInstanceSelectWrapper}
      instancesError={instancesError}
      onSpawnServer={handleSpawnServer}
      onTerminateServer={handleTerminateServer}
      onConnectToVpn={handleConnectToVpn}
      serverStatus={serverStatus}
      isSpawning={isSpawning}
      isTerminating={isTerminating}
      isConnecting={isConnecting}
      isLoading={isLoading}
      spawnError={spawnError}
      terminateError={terminateError}
      vpnError={vpnError}
      onNavigateToSettings={onNavigateToSettings}
    />
  );
}
