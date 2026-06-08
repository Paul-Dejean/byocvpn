import { useState, useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invokeCommand } from "../lib/invokeCommand";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  Region,
  CloudProviderName,
  Instance,
  InstanceState,
  SpawnJobState,
  JobStep,
  JobStepStatus,
} from "../types";

enum SpawnEvent {
  Progress = "spawn-progress",
  InstanceLaunched = "spawn-instance-launched",
  Complete = "spawn-complete",
  Failed = "spawn-failed",
}

interface SpawnJob {
  jobId: string;
  steps: JobStep[];
  region: string;
  provider: CloudProviderName;
}

interface SpawnProgressEvent {
  jobId: string;
  stepId: string;
  status: JobStepStatus;
  error?: string;
}

interface SpawnInstanceLaunchedEvent {
  jobId: string;
  instance: Instance;
}

interface SpawnCompleteEvent {
  jobId: string;
  instance: Instance;
}

interface ActiveSpawnJob extends SpawnJob {
  instanceId: string | null;
  stepStatuses: Record<string, JobStepStatus>;
}

interface RawInstanceData {
  fetched: Instance[];
  activeJobs: ActiveSpawnJob[];
}

async function fetchRawInstanceData(): Promise<RawInstanceData> {
  const [fetched, activeJobs] = await Promise.all([
    invokeCommand<Instance[]>("list_instances"),
    invokeCommand<ActiveSpawnJob[]>("list_active_spawn_jobs"),
  ]);
  return { fetched, activeJobs };
}

export function useInstances(_regions: Region[]) {
  const queryClient = useQueryClient();
  const [instances, setInstances] = useState<Instance[]>([]);
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<
    string | null
  >(null);
  const [error, setError] = useState<string | null>(null);
  const [spawnJobs, setSpawnJobs] = useState<Record<string, SpawnJobState>>({});
  const spawnJobsRef = useRef(spawnJobs);
  const instancesRef = useRef(instances);

  useEffect(() => {
    spawnJobsRef.current = spawnJobs;
  }, [spawnJobs]);

  useEffect(() => {
    instancesRef.current = instances;
  }, [instances]);

  const { data: rawData, isLoading, isFetching } = useQuery({
    queryKey: ["instances"],
    queryFn: fetchRawInstanceData,
    staleTime: 0,
  });

  useEffect(() => {
    if (!rawData) return;

    const { fetched, activeJobs } = rawData;
    const newSpawnJobs: Record<string, SpawnJobState> = {};
    const recoveredSpawningPlaceholders: Instance[] = [];

    for (const activeJob of activeJobs) {
      const existingLocalJob = spawnJobsRef.current[activeJob.jobId];
      const instanceId =
        existingLocalJob?.instanceId ??
        activeJob.instanceId ??
        `spawning-recovered-${activeJob.jobId}`;

      newSpawnJobs[activeJob.jobId] = {
        jobId: activeJob.jobId,
        instanceId,
        region: activeJob.region,
        provider: activeJob.provider,
        steps: activeJob.steps.map((step) => ({
          ...step,
          status: activeJob.stepStatuses[step.id] ?? JobStepStatus.Pending,
        })),
      };

      if (!activeJob.instanceId && !existingLocalJob) {
        recoveredSpawningPlaceholders.push({
          id: instanceId,
          name: "Deploying…",
          state: InstanceState.Spawning,
          publicIpV4: "",
          publicIpV6: "",
          region: activeJob.region,
          provider: activeJob.provider,
        });
      }
    }

    if (Object.keys(newSpawnJobs).length > 0) {
      setSpawnJobs((previous) => ({ ...previous, ...newSpawnJobs }));
    }

    const spawnIdsWithPlaceholder = new Set(
      [
        ...Object.values(newSpawnJobs),
        ...Object.values(spawnJobsRef.current),
      ]
        .filter((job) => job.instanceId.startsWith("spawning-"))
        .map((job) => job.jobId),
    );

    const sorted = [...fetched]
      .filter(
        (instance) =>
          !instance.spawnId ||
          !spawnIdsWithPlaceholder.has(instance.spawnId),
      )
      .sort((a, b) => {
        const installingA = a.state === InstanceState.Installing ? 0 : 1;
        const installingB = b.state === InstanceState.Installing ? 0 : 1;
        return installingA - installingB;
      });

    setInstances((previous) => {
      const recoveredIds = new Set(recoveredSpawningPlaceholders.map((p) => p.id));
      const localSpawningInstances = previous.filter(
        (instance) =>
          instance.id.startsWith("spawning-") && !recoveredIds.has(instance.id),
      );
      return [...recoveredSpawningPlaceholders, ...localSpawningInstances, ...sorted];
    });
  }, [rawData]);

  useEffect(() => {
    const progressUnlisten = listen<SpawnProgressEvent>(
      SpawnEvent.Progress,
      ({ payload }) => {
        const { jobId, stepId, status, error: stepError } = payload;
        setSpawnJobs((previous) => {
          const job = previous[jobId];
          if (!job) return previous;
          return {
            ...previous,
            [jobId]: {
              ...job,
              steps: job.steps.map((step) =>
                step.id === stepId
                  ? { ...step, status, error: stepError }
                  : step,
              ),
            },
          };
        });
      },
    );

    const launchedUnlisten = listen<SpawnInstanceLaunchedEvent>(
      SpawnEvent.InstanceLaunched,
      ({ payload }) => {
        const { jobId, instance } = payload;
        const job = spawnJobsRef.current[jobId];
        if (!job) return;
        setInstances((previous) =>
          previous.map((existing) =>
            existing.id === job.instanceId ? instance : existing,
          ),
        );
        setSpawnJobs((previous) => ({
          ...previous,
          [jobId]: { ...previous[jobId], instanceId: instance.id },
        }));
      },
    );

    const completeUnlisten = listen<SpawnCompleteEvent>(
      SpawnEvent.Complete,
      ({ payload }) => {
        const { jobId, instance } = payload;
        const job = spawnJobsRef.current[jobId];
        if (job) {
          setInstances((previous) =>
            previous.map((existing) =>
              existing.id === job.instanceId ? instance : existing,
            ),
          );
        }
        setSpawnJobs((previous) => {
          const { [jobId]: _, ...rest } = previous;
          return rest;
        });
        toast.success("Server deployed successfully!");
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      SpawnEvent.Failed,
      ({ payload }) => {
        const { jobId, error: failureError } = payload;
        const job = spawnJobsRef.current[jobId];
        if (job) {
          setInstances((previous) =>
            previous.map((existing) =>
              existing.id === job.instanceId
                ? {
                    ...existing,
                    state: InstanceState.Error,
                    errorReason: failureError,
                  }
                : existing,
            ),
          );
        }
        toast.error("Server deployment failed");
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      launchedUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const refetch = () =>
    queryClient.invalidateQueries({ queryKey: ["instances"] });

  const spawnInstance = async (
    regionName: string,
    provider: CloudProviderName,
  ): Promise<Instance> => {
    const tempId = `spawning-${Date.now()}`;

    const placeholder: Instance = {
      id: tempId,
      name: "Deploying…",
      state: InstanceState.Spawning,
      publicIpV4: "",
      publicIpV6: "",
      region: regionName,
      provider,
    };

    setInstances((previous) => [placeholder, ...previous]);

    try {
      const job = await invokeCommand<SpawnJob>("spawn_instance", {
        region: regionName,
        provider,
      });

      setSpawnJobs((previous) => ({
        ...previous,
        [job.jobId]: {
          jobId: job.jobId,
          instanceId: tempId,
          region: regionName,
          provider,
          steps: job.steps.map((step, index) => ({
            ...step,
            status:
              index === 0 ? JobStepStatus.Running : JobStepStatus.Pending,
          })),
        },
      }));
    } catch (spawnError) {
      setInstances((previous) =>
        previous.filter((instance) => instance.id !== tempId),
      );
      const message =
        spawnError instanceof Error
          ? spawnError.message
          : "Failed to start deployment";
      setError(message);
      toast.error(message);
    }

    return placeholder;
  };

  const terminateInstance = async (
    instanceId: string,
    region: string,
    provider: CloudProviderName,
  ): Promise<void> => {
    setTerminatingInstanceId(instanceId);
    setError(null);
    try {
      await invokeCommand("terminate_instance", {
        instanceId,
        region,
        provider,
      });
      setInstances((previous) =>
        previous.filter((instance) => instance.id !== instanceId),
      );
      toast.success("Server terminated successfully!");
    } catch (terminateError) {
      const message =
        terminateError instanceof Error
          ? terminateError.message
          : "Failed to terminate instance";
      setError(message);
      toast.error(message);
      console.error("Failed to terminate instance:", terminateError);
      throw terminateError;
    } finally {
      setTerminatingInstanceId(null);
    }
  };

  function getSpawnJobForInstance(
    instanceId: string,
  ): SpawnJobState | undefined {
    return Object.values(spawnJobs).find(
      (job) => job.instanceId === instanceId,
    );
  }

  const clearError = () => setError(null);

  const dismissFailedInstance = (instanceId: string) => {
    const job = Object.values(spawnJobs).find(
      (spawnJob) => spawnJob.instanceId === instanceId,
    );
    if (job) {
      setSpawnJobs((previous) => {
        const { [job.jobId]: _, ...rest } = previous;
        return rest;
      });
    }
    setInstances((previous) =>
      previous.filter((instance) => instance.id !== instanceId),
    );
  };

  return {
    instances,
    isLoading,
    isRefreshing: isFetching && !isLoading,
    isSpawning: Object.keys(spawnJobs).length > 0,
    terminatingInstanceId,
    error,
    spawnInstance,
    terminateInstance,
    clearError,
    dismissFailedInstance,
    refetch,
    getSpawnJobForInstance,
  };
}
