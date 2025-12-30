import { AwsRegion, ExistingInstance, RegionGroup } from "../../types";
import { SettingsButton } from "../settings/SettingsButton";
import { useState } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";
import { useRegions, useInstances, useVpnConnection } from "../../hooks";

interface ServerManagementViewProps {
  onNavigateToSettings?: () => void;
}

export function ServerManagementView({
  onNavigateToSettings,
}: ServerManagementViewProps) {
  const [showRegionSelector, setShowRegionSelector] = useState(false);
  const [localSelectedInstance, setLocalSelectedInstance] =
    useState<ExistingInstance | null>(null);
  const [spawningRegions, setSpawningRegions] = useState<string[]>([]);

  // Use all hooks needed for server management
  const {
    regions,
    groupedRegions,
    selectedRegion,
    isLoading: regionsLoading,
    error: regionsError,
    handleRegionSelect,
  } = useRegions();

  const {
    existingInstances,
    isLoading: instancesLoading,
    isSpawning,
    isTerminating,
    spawnInstance,
    terminateInstance,
    clearSelectedInstance,
  } = useInstances(regions);

  const {
    isConnecting,
    error: vpnError,
    handleConnectToVpn,
  } = useVpnConnection();

  // Use local state for selected instance
  const currentInstance = localSelectedInstance;

  const isLoading = regionsLoading || instancesLoading;

  const handleSelectInstance = (instance: ExistingInstance) => {
    setLocalSelectedInstance(instance);
    // Find and select the region for this instance
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === instance.region);
    if (region) handleRegionSelect(region);
    // Close region selector if it's open
    setShowRegionSelector(false);
  };

  const handleSelectRegion = async (region: AwsRegion) => {
    handleRegionSelect(region);
    clearSelectedInstance();
    setShowRegionSelector(false);

    // Add region to spawning list
    setSpawningRegions((prev) => [...prev, region.name]);

    // Spawn server in selected region
    try {
      const serverDetails = await spawnInstance(region.name);
      // Instance is automatically added to list by the hook
      setLocalSelectedInstance({
        id: serverDetails.instance_id,
        name: "VPN Server",
        state: "running",
        public_ip_v4: serverDetails.public_ip_v4,
        public_ip_v6: serverDetails.public_ip_v6 || "",
        region: region.name,
      });
    } catch (error) {
      console.error("Failed to spawn server:", error);
    } finally {
      // Remove region from spawning list
      setSpawningRegions((prev) => prev.filter((r) => r !== region.name));
    }
  };

  const handleTerminateServer = async () => {
    if (!currentInstance) return;

    try {
      await terminateInstance(currentInstance.id, currentInstance.region || "");
      // Instance is automatically removed from list by the hook
      setLocalSelectedInstance(null);
    } catch (error) {
      console.error("Failed to terminate server:", error);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white overflow-hidden">
      {showRegionSelector ? (
        /* Full-screen Region Selector View */
        <RegionSelector
          groupedRegions={groupedRegions}
          existingInstances={existingInstances}
          onSelectRegion={handleSelectRegion}
          onClose={() => setShowRegionSelector(false)}
        />
      ) : (
        <>
          {/* Header */}
          <div className="bg-gray-800 p-6 border-b border-gray-700 flex-shrink-0">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-3xl font-bold mb-2 text-blue-400">
                  VPN Server Management
                </h1>
                <p className="text-gray-300">
                  Select a region and manage your servers
                </p>
              </div>
              <SettingsButton onClick={() => onNavigateToSettings?.()} />
            </div>
          </div>

          {/* Two-Panel Layout */}
          <div className="flex-1 flex min-h-0">
            {/* Left Panel: Server List */}
            <ServerList
              instances={existingInstances}
              selectedInstance={currentInstance}
              groupedRegions={groupedRegions}
              isLoading={isLoading}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => setShowRegionSelector(true)}
              spawningRegions={spawningRegions}
            />

            {/* Right Panel: Dynamic Content */}
            {currentInstance ? (
              <ServerDetails
                instance={currentInstance}
                groupedRegions={groupedRegions}
                isConnecting={isConnecting}
                isTerminating={isTerminating}
                vpnError={vpnError}
                onConnect={(data) =>
                  handleConnectToVpn({
                    instance_id: data.instance_id,
                    public_ip_v4: data.public_ip_v4,
                    public_ip_v6: data.public_ip_v6,
                    region: data.region || "",
                    client_private_key: data.client_private_key,
                    server_public_key: data.server_public_key,
                  })
                }
                onTerminate={handleTerminateServer}
              />
            ) : (
              <EmptyState
                title="Select a server"
                description="Choose a server from the left to view details"
              />
            )}
          </div>
        </>
      )}
    </div>
  );
}
