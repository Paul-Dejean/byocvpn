import { Instance, InstanceState } from "../../types";
import { useState, useEffect } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";
import { ProviderSelector } from "../providers/ProviderSelector";
import { CloudProviderName } from "../../types";

import { useInstancesContext, useRegionsContext } from "../../contexts";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";

type CreationStep = "idle" | "selecting-provider" | "selecting-region";

export function ServerManagementView() {
  const [creationStep, setCreationStep] = useState<CreationStep>("idle");
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName>(CloudProviderName.Aws);

  const [selectedInstance, setSelectedInstance] = useState<Instance | null>(
    null,
  );

  const { groupedRegions, isLoading: regionsLoading } = useRegionsContext();
  const {
    instances,
    isLoading: instancesLoading,
    isRefreshing,
    terminatingInstanceId,
    terminateInstance,
    dismissFailedInstance,
    getSpawnJobForInstance,
  } = useInstancesContext();

  const {
    isConnecting,
    error: vpnError,
    connectToVpn,
  } = useVpnConnectionContext();

  const isLoading = regionsLoading || instancesLoading;

  useEffect(() => {
    if (!selectedInstance) return;
    const live = instances.find((i) => i.id === selectedInstance.id);
    if (live) {

      if (live !== selectedInstance) setSelectedInstance(live);
    } else if (selectedInstance.state === InstanceState.Spawning) {

      const replacement = instances.find(
        (i) =>
          i.region === selectedInstance.region &&
          i.provider === selectedInstance.provider &&
          i.state !== InstanceState.Spawning,
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

    try {
      await terminateInstance(
        selectedInstance.id,
        selectedInstance.region || "",
        selectedInstance.provider || CloudProviderName.Aws,
      );

      setSelectedInstance(null);
    } catch (error) {
      console.error("Failed to terminate server:", error);
    }
  };

  const onDismiss = () => {
    if (!selectedInstance) return;
    dismissFailedInstance(selectedInstance.id);
    setSelectedInstance(null);
  };

  const handleSelectProvider = (provider: CloudProviderName) => {
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
          <div className="flex-1 flex min-h-0">
            <ServerList
              instances={instances}
              selectedInstance={selectedInstance}
              groupedRegions={groupedRegions}
              isLoading={isLoading}
              isRefreshing={isRefreshing}
              getSpawnJobForInstance={getSpawnJobForInstance}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => setCreationStep("selecting-provider")}
            />

            {selectedInstance ? (
              <ServerDetails
                instance={selectedInstance}
                isConnecting={isConnecting}
                isTerminating={terminatingInstanceId === selectedInstance?.id}
                vpnError={vpnError}
                spawnJob={getSpawnJobForInstance(selectedInstance.id)}
                onConnect={onConnect}
                onTerminate={onTerminate}
                onDismiss={onDismiss}
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
