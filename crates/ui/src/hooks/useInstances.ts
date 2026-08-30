import { useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { SpawnInstanceEvent, commands } from "../bindings";
import { CloudProviderName, Instance, InstanceState, SpawnJob } from "../types";
import { Result } from "../lib/result";
import { instancesQueryOptions } from "../queries/instances";
import { spawnJobsQueryOptions } from "../queries/spawnJobs";

const SpawnEventKind = {
  Started: "STARTED",
  Progress: "PROGRESS",
  InstanceLaunched: "INSTANCE_LAUNCHED",
  Complete: "COMPLETE",
  Failed: "FAILED",
} as const satisfies Record<string, SpawnInstanceEvent["kind"]>;

export function useInstances() {
  const queryClient = useQueryClient();
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<
    string | null
  >(null);

  const {
    data: fetchedInstances = [],
    isLoading,
    isFetching,
  } = useQuery(instancesQueryOptions);

  const { data: spawnJobs = [] } = useQuery(spawnJobsQueryOptions);

  const instances = useMemo(
    () => [...fetchedInstances].sort(compareByDeploymentProgress),
    [fetchedInstances],
  );

  const pendingSpawnJobs = useMemo(
    () =>
      spawnJobs.filter(
        (spawnJob) =>
          !spawnJob.instanceId ||
          !instances.some((instance) => instance.id === spawnJob.instanceId),
      ),
    [spawnJobs, instances],
  );

  async function startSpawnJob(
    region: string,
    provider: CloudProviderName,
  ): Promise<SpawnJob | null> {
    let resolveStartedJob!: (job: SpawnJob) => void;
    const startedJob = new Promise<SpawnJob>((resolve) => {
      resolveStartedJob = resolve;
    });

    const onEvent = new Channel<SpawnInstanceEvent>();
    onEvent.onmessage = (event) => {
      if (event.kind === SpawnEventKind.Started) {
        resolveStartedJob(event.job);
      }
      applySpawnEvent(event);
    };

    const result = await commands.spawnInstance(region, provider, onEvent);
    if (result.status === "error") {
      toast.error(result.error);
      return null;
    }
    return await startedJob;
  }

  function applySpawnEvent(event: SpawnInstanceEvent) {
    switch (event.kind) {
      case SpawnEventKind.Started:
        queryClient.setQueryData(
          spawnJobsQueryOptions.queryKey,
          (previous = []) => [
            ...previous.filter((job) => job.jobId !== event.job.jobId),
            event.job,
          ],
        );
        return;

      case SpawnEventKind.Progress:
        queryClient.setQueryData(
          spawnJobsQueryOptions.queryKey,
          (previous = []) =>
            previous.map((job) => ({
              ...job,
              steps: job.steps.map((step) =>
                step.id === event.stepId
                  ? { ...step, status: event.status, error: event.error }
                  : step,
              ),
            })),
        );
        return;

      case SpawnEventKind.InstanceLaunched:
        refetchSpawnJobs();
        refetchInstances();
        return;

      case SpawnEventKind.Complete:
        refetchSpawnJobs();
        refetchInstances();
        toast.success("Server deployed successfully!");
        return;

      case SpawnEventKind.Failed:
        refetchSpawnJobs();
        toast.error("Server deployment failed");
        return;
    }
  }

  async function dismissSpawnJob(jobId: string): Promise<void> {
    const result = await commands.dismissSpawnJob(jobId);
    if (result.status === "error") {
      toast.error(result.error);
      return;
    }
    await queryClient.invalidateQueries({
      queryKey: spawnJobsQueryOptions.queryKey,
    });
  }

  async function terminateInstance(
    instanceId: string,
    region: string,
    provider: CloudProviderName,
  ): Promise<Result<string>> {
    setTerminatingInstanceId(instanceId);
    const result = await commands.terminateInstance(
      instanceId,
      region,
      provider,
    );
    if (result.status === "error") {
      setTerminatingInstanceId(null);
      toast.error(result.error);
      return result;
    }
    queryClient.setQueryData(instancesQueryOptions.queryKey, (previous = []) =>
      previous.filter((instance) => instance.id !== instanceId),
    );
    setTerminatingInstanceId(null);
    toast.success("Server terminated successfully!");
    refetchInstances();
    return result;
  }

  async function refetchInstances(): Promise<void> {
    await queryClient.invalidateQueries({
      queryKey: instancesQueryOptions.queryKey,
    });
  }

  function refetchSpawnJobs() {
    queryClient.invalidateQueries({
      queryKey: spawnJobsQueryOptions.queryKey,
    });
  }

  return {
    instances,
    spawnJobs,
    pendingSpawnJobs,
    isLoading,
    isRefreshing: isFetching && !isLoading,
    terminatingInstanceId,
    startSpawnJob,
    dismissSpawnJob,
    terminateInstance,
    refetchInstances,
  };
}

function compareByDeploymentProgress(first: Instance, second: Instance) {
  const firstRank = first.state === InstanceState.Installing ? 0 : 1;
  const secondRank = second.state === InstanceState.Installing ? 0 : 1;
  return firstRank - secondRank;
}
