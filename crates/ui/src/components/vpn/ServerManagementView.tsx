import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { CloudProviderName, Instance, SpawnJob } from "../../types";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { SpawnJobDetails } from "../jobs/SpawnJobDetails";
import { EmptyState } from "../primitives/EmptyState";
import { ProviderSelector } from "../providers/ProviderSelector";

import { useInstances } from "../../hooks/useInstances";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { configuredProvidersQueryOptions } from "../../queries/configuredProviders";

enum CreationStep {
  Idle = "IDLE",
  SelectingProvider = "SELECTING_PROVIDER",
  SelectingRegion = "SELECTING_REGION",
}

enum SelectionKind {
  SpawnJob = "SPAWN_JOB",
  Instance = "INSTANCE",
}

type Selection =
  | { kind: SelectionKind.SpawnJob; jobId: string }
  | { kind: SelectionKind.Instance; instanceId: string };

export function ServerManagementView() {
  const [creationStep, setCreationStep] = useState<CreationStep>(
    CreationStep.Idle,
  );
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName>(
    CloudProviderName.Aws,
  );
  const [selection, setSelection] = useState<Selection | null>(null);

  const queryClient = useQueryClient();
  const {
    instances,
    spawnJobs,
    pendingSpawnJobs,
    isLoading,
    isRefreshing,
    terminatingInstanceId,
    terminateInstance,
    dismissSpawnJob,
  } = useInstances();

  const {
    isConnecting,
    error: vpnError,
    connectToVpn,
    clearError,
  } = useVpnConnectionContext();

  const selectedSpawnJob =
    selection?.kind === SelectionKind.SpawnJob
      ? spawnJobs.find((spawnJob) => spawnJob.jobId === selection.jobId)
      : undefined;

  const selectedInstance =
    (selection?.kind === SelectionKind.Instance
      ? instances.find((instance) => instance.id === selection.instanceId)
      : instances.find(
          (instance) => instance.spawnId === selection?.jobId,
        )) ?? null;

  function handleSelectInstance(instance: Instance) {
    setSelection({ kind: SelectionKind.Instance, instanceId: instance.id });
    clearError();
  }

  function handleSelectSpawnJob(spawnJob: SpawnJob) {
    setSelection({ kind: SelectionKind.SpawnJob, jobId: spawnJob.jobId });
    clearError();
  }

  async function handleConnect(instance: Instance) {
    await connectToVpn(instance);
  }

  async function handleTerminate() {
    if (!selectedInstance) return;

    const result = await terminateInstance(
      selectedInstance.id,
      selectedInstance.region,
      selectedInstance.provider,
    );
    if (result.status === "ok") {
      setSelection(null);
    }
  }

  async function handleDismissSpawnJob(jobId: string) {
    await dismissSpawnJob(jobId);
    setSelection(null);
  }

  async function handleAddNewServer() {
    const configuredProviders = await queryClient.ensureQueryData(
      configuredProvidersQueryOptions,
    );
    if (configuredProviders.length === 1) {
      setSelectedProvider(configuredProviders[0]);
      setCreationStep(CreationStep.SelectingRegion);
    } else {
      setCreationStep(CreationStep.SelectingProvider);
    }
  }

  return (
    <div className="flex flex-col h-full bg-gray-900 text-primary overflow-hidden">
      {creationStep === CreationStep.SelectingProvider ? (
        <ProviderSelector
          onSelectProvider={(provider) => {
            setSelectedProvider(provider);
            setCreationStep(CreationStep.SelectingRegion);
          }}
          onClose={() => setCreationStep(CreationStep.Idle)}
        />
      ) : creationStep === CreationStep.SelectingRegion ? (
        <RegionSelector
          provider={selectedProvider}
          onClose={() => setCreationStep(CreationStep.Idle)}
          onSpawnStarted={(jobId) => {
            setSelection({ kind: SelectionKind.SpawnJob, jobId });
            setCreationStep(CreationStep.Idle);
          }}
        />
      ) : (
        <div className="flex-1 flex min-h-0">
          <ServerList
            instances={instances}
            spawnJobs={pendingSpawnJobs}
            selectedInstanceId={selectedInstance?.id ?? null}
            selectedJobId={selectedSpawnJob?.jobId ?? null}
            isLoading={isLoading}
            isRefreshing={isRefreshing}
            onSelectInstance={handleSelectInstance}
            onSelectSpawnJob={handleSelectSpawnJob}
            onAddNewServer={handleAddNewServer}
          />

          {selectedInstance ? (
            <ServerDetails
              key={selectedInstance.id}
              instance={selectedInstance}
              isConnecting={isConnecting}
              isTerminating={terminatingInstanceId === selectedInstance.id}
              vpnError={vpnError}
              spawnJob={spawnJobs.find(
                (spawnJob) => spawnJob.jobId === selectedInstance.spawnId,
              )}
              onConnect={handleConnect}
              onTerminate={handleTerminate}
            />
          ) : selectedSpawnJob ? (
            <SpawnJobDetails
              spawnJob={selectedSpawnJob}
              onDismiss={handleDismissSpawnJob}
            />
          ) : (
            <EmptyState
              title="Select a server"
              description="Choose a server from the left to view details"
            />
          )}
        </div>
      )}
    </div>
  );
}
