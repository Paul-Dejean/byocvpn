import { Instance } from "../../types";
import { useState, useEffect } from "react";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { EmptyState } from "../common/EmptyState";
import { ProviderSelector } from "../providers/ProviderSelector";
import { useCredentials } from "../../hooks/useCredentials";

import { useInstancesContext, useRegionsContext } from "../../contexts";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";

type CreationStep = "idle" | "selecting-provider" | "selecting-region";

interface ServerManagementViewProps {
  onNavigateToAddAccount: () => void;
}

export function ServerManagementView({
  onNavigateToAddAccount,
}: ServerManagementViewProps) {
  const [creationStep, setCreationStep] = useState<CreationStep>("idle");
  const [selectedProvider, setSelectedProvider] = useState<string>("aws");
  const [hasAnyAccount, setHasAnyAccount] = useState<boolean | null>(null);

  const [selectedInstance, setSelectedInstance] = useState<Instance | null>(
    null,
  );

  const { groupedRegions, isLoading: regionsLoading } = useRegionsContext();
  const { loadCredentials } = useCredentials();

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

  useEffect(() => {
    const checkAnyAccount = async () => {
      for (const provider of ["aws", "oracle", "gcp", "azure"] as const) {
        const existing = await loadCredentials(provider);
        if (existing !== null) {
          setHasAnyAccount(true);
          return;
        }
      }
      setHasAnyAccount(false);
    };
    checkAnyAccount();
  }, []);

  useEffect(() => {
    if (!selectedInstance) return;
    const live = instances.find((i) => i.id === selectedInstance.id);
    if (live) {

      if (live !== selectedInstance) setSelectedInstance(live);
    } else if (selectedInstance.state === "spawning") {

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

      setSelectedInstance(null);
    } catch (error) {
      console.error("Failed to terminate server:", error);
    }
  };

  const handleSelectProvider = (provider: string) => {
    setSelectedProvider(provider);
    setCreationStep("selecting-region");
  };

  if (hasAnyAccount === false) {
    return (
      <div className="flex flex-col h-full bg-gray-900 text-white overflow-hidden">
        <div className="flex-1 flex flex-col items-center justify-center gap-6 p-8">
          <div className="w-20 h-20 rounded-2xl bg-blue-600/10 border border-blue-600/20 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-10 w-10 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
            </svg>
          </div>
          <div className="text-center">
            <h2 className="text-2xl font-bold text-white mb-2">No cloud account connected</h2>
            <p className="text-gray-400 max-w-sm">
              Connect a cloud account to start deploying VPN servers. Supports AWS, Oracle Cloud, GCP, and Azure.
            </p>
          </div>
          <button
            onClick={onNavigateToAddAccount}
            className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-xl font-semibold transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            Add Account
          </button>
        </div>
      </div>
    );
  }

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
            {}
            <ServerList
              instances={instances}
              selectedInstance={selectedInstance}
              groupedRegions={groupedRegions}
              isLoading={isLoading}
              getSpawnJobForInstance={getSpawnJobForInstance}
              onSelectInstance={handleSelectInstance}
              onAddNewServer={() => setCreationStep("selecting-provider")}
            />

            {}
            {selectedInstance ? (
              <ServerDetails
                instance={selectedInstance}
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
