import { Instance } from "../../types";
import { SettingsButton } from "../settings/SettingsButton";
import { useState } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";
import { useVpnConnection } from "../../hooks";
import { useInstancesContext, useRegionsContext } from "../../contexts";

interface ServerManagementViewProps {
  selectedInstance?: Instance | null;
  setSelectedInstance?: (instance: Instance | null) => void;
  onNavigateToSettings?: () => void;
}

export function ServerManagementView({
  onNavigateToSettings,
}: ServerManagementViewProps) {
  const [isInstanceCreationFormVisible, showInstanceCreationForm] =
    useState(false);

  const [selectedInstance, setSelectedInstance] = useState<Instance | null>(
    null
  );

  // Use all hooks needed for server management
  const { groupedRegions, isLoading: regionsLoading } = useRegionsContext();

  const {
    instances,
    isLoading: instancesLoading,
    isTerminating,
    terminateInstance,
  } = useInstancesContext();

  const {
    vpnStatus,
    isConnecting,
    error: vpnError,
    connectToVpn,
  } = useVpnConnection();

  const isLoading = regionsLoading || instancesLoading;

  const connectedInstance =
    vpnStatus.connected === true ? vpnStatus.instance : null;

  const handleSelectInstance = (instance: Instance) => {
    setSelectedInstance(instance);
  };

  const handleTerminateServer = async () => {
    if (!selectedInstance) return;
    console.log({ selectedInstance });

    try {
      await terminateInstance(
        selectedInstance.id,
        selectedInstance.region || ""
      );
      // Instance is automatically removed from list by the hook
      setSelectedInstance(null);
    } catch (error) {
      console.error("Failed to terminate server:", error);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white overflow-hidden">
      {isInstanceCreationFormVisible ? (
        /* Full-screen Region Selector View */
        <RegionSelector onClose={() => showInstanceCreationForm(false)} />
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
              instances={instances}
              selectedInstance={selectedInstance}
              groupedRegions={groupedRegions}
              isLoading={isLoading}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => showInstanceCreationForm(true)}
            />

            {/* Right Panel: Dynamic Content */}
            {selectedInstance ? (
              <ServerDetails
                instance={selectedInstance}
                groupedRegions={groupedRegions}
                isConnecting={isConnecting}
                isTerminating={isTerminating}
                vpnError={vpnError}
                onConnect={(instance) => connectToVpn(instance)}
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
