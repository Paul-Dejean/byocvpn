import { AwsRegion, ExistingInstance, RegionGroup } from "../../types";
import { SettingsButton } from "../settings/SettingsButton";
import { useState } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";

interface ServerManagementViewProps {
  // Regions
  groupedRegions: RegionGroup[];
  selectedRegion: AwsRegion | null;
  onRegionSelect: (region: AwsRegion) => void;
  regionsError: string | null;

  // Instances
  existingInstances: ExistingInstance[];
  selectedInstance: any;
  onInstanceSelect: (instance: ExistingInstance) => void;
  instancesError: string | null;

  // Actions
  onSpawnServer: () => void;
  onTerminateServer: () => void;
  onConnectToVpn: (instance: any) => void;

  // State
  serverStatus: string;
  isSpawning: boolean;
  isTerminating: boolean;
  isConnecting: boolean;
  isLoading: boolean;

  // Errors
  spawnError: string | null;
  terminateError: string | null;
  vpnError: string | null;

  onNavigateToSettings?: () => void;
}

export function ServerManagementView({
  groupedRegions,
  selectedRegion,
  onRegionSelect,
  regionsError,
  existingInstances,
  selectedInstance,
  onInstanceSelect,
  instancesError,
  onSpawnServer,
  onTerminateServer,
  onConnectToVpn,
  serverStatus,
  isSpawning,
  isTerminating,
  isConnecting,
  isLoading,
  spawnError,
  terminateError,
  vpnError,
  onNavigateToSettings,
}: ServerManagementViewProps) {
  const [showRegionSelector, setShowRegionSelector] = useState(false);
  const [localSelectedInstance, setLocalSelectedInstance] =
    useState<ExistingInstance | null>(null);

  // Use local state if parent doesn't provide selectedInstance
  const currentInstance = selectedInstance || localSelectedInstance;

  const handleSelectInstance = (instance: ExistingInstance) => {
    setLocalSelectedInstance(instance);
    onInstanceSelect(instance);
    // Find and select the region for this instance
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === instance.region);
    if (region) onRegionSelect(region);
    // Close region selector if it's open
    setShowRegionSelector(false);
  };

  const handleSelectRegion = (region: AwsRegion) => {
    onRegionSelect(region);
    onSpawnServer();
    setShowRegionSelector(false);
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white overflow-hidden">
      {showRegionSelector ? (
        /* Full-screen Region Selector View */
        <RegionSelector
          groupedRegions={groupedRegions}
          existingInstances={existingInstances}
          isSpawning={isSpawning}
          spawnError={spawnError}
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
              error={instancesError}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => setShowRegionSelector(true)}
              isSpawning={isSpawning}
              spawningRegion={selectedRegion?.name}
            />

            {/* Right Panel: Dynamic Content */}
            {currentInstance ? (
              <ServerDetails
                instance={currentInstance}
                groupedRegions={groupedRegions}
                isConnecting={isConnecting}
                isTerminating={isTerminating}
                vpnError={vpnError}
                terminateError={terminateError}
                onConnect={onConnectToVpn}
                onTerminate={(instance) => {
                  onInstanceSelect(instance);
                  onTerminateServer();
                }}
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
