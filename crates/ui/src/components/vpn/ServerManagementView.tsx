import { useState } from "react";
import { CloudProviderName, Instance, SpawnJob } from "../../types";
import { ServerList } from "../servers/ServerList";
import { RegionSelector } from "../regions/RegionSelector";
import { ServerDetails } from "../servers/ServerDetails";
import { SpawnJobDetails } from "../jobs/SpawnJobDetails";
import { EmptyState } from "../primitives/EmptyState";
import { ProviderSelector } from "../providers/ProviderSelector";

import { useInstancesContext, useRegionsContext } from "../../contexts";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";

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

  const { groupedRegions, isLoading: regionsLoading } = useRegionsContext();
  const {
    instances,
    spawnJobs,
    isLoading: instancesLoading,
    isRefreshing,
    terminatingInstanceId,
    terminateInstance,
    dismissSpawnJob,
  } = useInstancesContext();

  const {
    isConnecting,
    error: vpnError,
    connectToVpn,
    clearError,
  } = useVpnConnectionContext();

  const isLoading = regionsLoading || instancesLoading;

  const pendingSpawnJobs = spawnJobs.filter(
    (spawnJob) =>
      !spawnJob.instanceId ||
      !instances.some((instance) => instance.id === spawnJob.instanceId),
  );

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

    try {
      await terminateInstance(
        selectedInstance.id,
        selectedInstance.region,
        selectedInstance.provider,
      );
      setSelection(null);
    } catch (terminateError) {
      console.error("Failed to terminate server:", terminateError);
    }
  }

  async function handleDismissSpawnJob(jobId: string) {
    await dismissSpawnJob(jobId);
    setSelection(null);
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
            groupedRegions={groupedRegions}
            isLoading={isLoading}
            isRefreshing={isRefreshing}
            onSelectInstance={handleSelectInstance}
            onSelectSpawnJob={handleSelectSpawnJob}
            onAddNewServer={() =>
              setCreationStep(CreationStep.SelectingProvider)
            }
          />

          {selectedInstance ? (
            <ServerDetails
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
