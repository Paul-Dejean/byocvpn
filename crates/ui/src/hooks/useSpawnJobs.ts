import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { CloudProviderName, Instance, JobStepStatus, SpawnJob } from "../types";
import { invokeCommand } from "../lib/invokeCommand";

export const SPAWN_JOBS_QUERY_KEY = ["spawn-jobs"];
export const INSTANCES_QUERY_KEY = ["instances"];

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

function fetchSpawnJobs(): Promise<SpawnJob[]> {
  return invokeCommand<SpawnJob[]>("list_active_spawn_jobs");
}

export function useSpawnJobs() {
  const queryClient = useQueryClient();

  const { data: spawnJobs = [] } = useQuery({
    queryKey: SPAWN_JOBS_QUERY_KEY,
    queryFn: fetchSpawnJobs,
    staleTime: 0,
    refetchOnReconnect: false,
    refetchInterval: (query) =>
      query.state.data?.length ? RUNNING_JOB_POLL_INTERVAL_MS : false,
  });

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
    } catch (spawnError) {
      const message =
        spawnError instanceof Error
          ? spawnError.message
          : "Failed to start deployment";
      toast.error(message);
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

  function refetchSpawnJobs() {
    queryClient.invalidateQueries({ queryKey: SPAWN_JOBS_QUERY_KEY });
  }

  function refetchInstances() {
    queryClient.invalidateQueries({ queryKey: INSTANCES_QUERY_KEY });
  }

  async function dismissSpawnJob(jobId: string): Promise<void> {
    await invokeCommand("dismiss_spawn_job", { jobId });
    await queryClient.invalidateQueries({ queryKey: SPAWN_JOBS_QUERY_KEY });
  }

  return { spawnJobs, startSpawnJob, dismissSpawnJob };
}
