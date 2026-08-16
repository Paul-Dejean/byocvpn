import { useEffect, useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  CloudProviderName,
  Instance,
  InstanceState,
  JobStepStatus,
  SpawnJob,
} from "../types";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { invokeCommand } from "../lib/invokeCommand";

const INSTANCES_QUERY_KEY = ["instances"];
const SPAWN_JOBS_QUERY_KEY = ["spawn-jobs"];
const AUTO_TERMINATED_EVENT = "instance-auto-terminated";
const RUNNING_JOB_POLL_INTERVAL_MS = 3000;

enum SpawnEventKind {
  Started = "STARTED",
  Progress = "PROGRESS",
  InstanceLaunched = "INSTANCE_LAUNCHED",
  Complete = "COMPLETE",
  Failed = "FAILED",
}

type SpawnInstanceEvent =
  | { kind: SpawnEventKind.Started; job: SpawnJob }
  | {
      kind: SpawnEventKind.Progress;
      stepId: string;
      status: JobStepStatus;
      error?: string;
    }
  | { kind: SpawnEventKind.InstanceLaunched; instance: Instance }
  | { kind: SpawnEventKind.Complete; instance: Instance }
  | { kind: SpawnEventKind.Failed; error: string };

export function useInstances() {
  const queryClient = useQueryClient();
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<
    string | null
  >(null);

  const {
    data: fetchedInstances = [],
    isLoading,
    isFetching,
  } = useQuery({
    queryKey: INSTANCES_QUERY_KEY,
    queryFn: () => invokeCommand<Instance[]>("list_instances"),
    staleTime: 0,
    refetchOnReconnect: false,
  });

  const { data: spawnJobs = [] } = useQuery({
    queryKey: SPAWN_JOBS_QUERY_KEY,
    queryFn: () => invokeCommand<SpawnJob[]>("list_active_spawn_jobs"),
    staleTime: 0,
    refetchOnReconnect: false,
    refetchInterval: (query) =>
      query.state.data?.length ? RUNNING_JOB_POLL_INTERVAL_MS : false,
  });

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

  useEffect(() => {
    const autoTerminatedUnlisten = listen(AUTO_TERMINATED_EVENT, () => {
      queryClient.invalidateQueries({ queryKey: INSTANCES_QUERY_KEY });
    });

    return () => {
      autoTerminatedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

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

    try {
      await invokeCommand("spawn_instance", { region, provider, onEvent });
      return await startedJob;
    } catch (error) {
      toast.error(extractErrorMessage(error, "Failed to start deployment"));
      return null;
    }
  }

  function applySpawnEvent(event: SpawnInstanceEvent) {
    switch (event.kind) {
      case SpawnEventKind.Started:
        queryClient.setQueryData<SpawnJob[]>(
          SPAWN_JOBS_QUERY_KEY,
          (previous = []) => [
            ...previous.filter((job) => job.jobId !== event.job.jobId),
            event.job,
          ],
        );
        return;

      case SpawnEventKind.Progress:
        queryClient.setQueryData<SpawnJob[]>(
          SPAWN_JOBS_QUERY_KEY,
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
    await invokeCommand("dismiss_spawn_job", { jobId });
    await queryClient.invalidateQueries({ queryKey: SPAWN_JOBS_QUERY_KEY });
  }

  async function terminateInstance(
    instanceId: string,
    region: string,
    provider: CloudProviderName,
  ): Promise<void> {
    setTerminatingInstanceId(instanceId);
    try {
      await invokeCommand("terminate_instance", {
        instanceId,
        region,
        provider,
      });
      await refetchInstances();
      toast.success("Server terminated successfully!");
    } catch (error) {
      toast.error(extractErrorMessage(error, "Failed to terminate instance"));
      throw error;
    } finally {
      setTerminatingInstanceId(null);
    }
  }

  async function refetchInstances(): Promise<void> {
    await queryClient.invalidateQueries({ queryKey: INSTANCES_QUERY_KEY });
  }

  function refetchSpawnJobs() {
    queryClient.invalidateQueries({ queryKey: SPAWN_JOBS_QUERY_KEY });
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
