import { Instance } from "../../types";
import { SettingsButton } from "../settings/SettingsButton";
import { useState, useEffect } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";
import { ProviderSelector } from "../providers/ProviderSelector";

import { useInstancesContext, useRegionsContext } from "../../contexts";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";

type CreationStep = "idle" | "selecting-provider" | "selecting-region";

interface ServerManagementViewProps {
  onNavigateToSettings: () => void;
}

export function ServerManagementView({
  onNavigateToSettings,
}: ServerManagementViewProps) {
  const [creationStep, setCreationStep] = useState<CreationStep>("idle");
  const [selectedProvider, setSelectedProvider] = useState<string>("aws");

  const [selectedInstance, setSelectedInstance] = useState<Instance | null>(
    null,
  );

  // Use all hooks needed for server management
  const { groupedRegions, isLoading: regionsLoading } = useRegionsContext();

  const {
    instances,
    isLoading: instancesLoading,
    terminatingInstanceId,
    terminateInstance,
    getSpawnJobForInstance,
  } = useInstancesContext();

  const {
    isConnecting,
    error: vpnError,
    connectToVpn,
  } = useVpnConnectionContext();

  const isLoading = regionsLoading || instancesLoading;

  // Keep selectedInstance in sync with the live instances array so that
  // state/IP updates (and the placeholder → real instance transition on
  // spawn-complete) are reflected automatically.
  useEffect(() => {
    if (!selectedInstance) return;
    const live = instances.find((i) => i.id === selectedInstance.id);
    if (live) {
      // Same ID — update in case state or IP changed.
      if (live !== selectedInstance) setSelectedInstance(live);
    } else if (selectedInstance.state === "spawning") {
      // The placeholder was replaced by the real instance (different ID).
      // Find it by matching region + provider with a non-spawning state.
      const replacement = instances.find(
        (i) =>
          i.region === selectedInstance.region &&
          i.provider === selectedInstance.provider &&
          i.state !== "spawning",
      );
      if (replacement) setSelectedInstance(replacement);
    }
  }, [instances]);

  const handleSelectInstance = (instance: Instance) => {
    setSelectedInstance(instance);
  };
  const onConnect = async (instance: Instance) => {
    await connectToVpn(instance);
  };

  const onTerminate = async () => {
    if (!selectedInstance) return;
    console.log({ selectedInstance });

    try {
      await terminateInstance(
        selectedInstance.id,
        selectedInstance.region || "",
        selectedInstance.provider || "aws",
      );
      // Instance is automatically removed from list by the hook
      setSelectedInstance(null);
    } catch (error) {
      console.error("Failed to terminate server:", error);
    }
  };

  const handleSelectProvider = (provider: string) => {
    setSelectedProvider(provider);
    setCreationStep("selecting-region");
  };

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white overflow-hidden">
      {creationStep === "selecting-provider" ? (
        <ProviderSelector
          onSelectProvider={handleSelectProvider}
          onClose={() => setCreationStep("idle")}
        />
      ) : creationStep === "selecting-region" ? (
        /* Full-screen Region Selector View */
        <RegionSelector
          provider={selectedProvider}
          onClose={() => setCreationStep("idle")}
          onSpawned={(instance) => {
            setSelectedInstance(instance);
            setCreationStep("idle");
          }}
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
              <SettingsButton onClick={() => onNavigateToSettings()} />
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
              getSpawnJobForInstance={getSpawnJobForInstance}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => setCreationStep("selecting-provider")}
            />

            {/* Right Panel: Dynamic Content */}
            {selectedInstance ? (
              <ServerDetails
                instance={selectedInstance}
                groupedRegions={groupedRegions}
                isConnecting={isConnecting}
                isTerminating={terminatingInstanceId === selectedInstance?.id}
                vpnError={vpnError}
                spawnJob={getSpawnJobForInstance(selectedInstance.id)}
                onConnect={onConnect}
                onTerminate={onTerminate}
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
