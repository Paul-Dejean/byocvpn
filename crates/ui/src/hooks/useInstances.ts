import { useState, useEffect, useRef } from "react";
import { invokeCommand } from "../lib/invokeCommand";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  Region,
  CloudProviderName,
  Instance,
  InstanceState,
  SpawnJobState,
  SpawnStep,
  SpawnStepStatus,
} from "../types";

enum SpawnEvent {
  Progress = "spawn-progress",
  InstanceLaunched = "spawn-instance-launched",
  Complete = "spawn-complete",
  Failed = "spawn-failed",
}

interface SpawnJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: CloudProviderName;
}

interface SpawnProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
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
  stepStatuses: Record<string, SpawnStepStatus>;
}

export function useInstances(regions: Region[]) {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<
    string | null
  >(null);
  const [error, setError] = useState<string | null>(null);
  const [spawnJobs, setSpawnJobs] = useState<Record<string, SpawnJobState>>({});
  const spawnJobsRef = useRef(spawnJobs);

  useEffect(() => {
    spawnJobsRef.current = spawnJobs;
  }, [spawnJobs]);

  useEffect(() => {
    if (regions.length > 0) fetchInstances();
  }, [regions]);

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
            previous.filter((existing) => existing.id !== job.instanceId),
          );
        }
        setSpawnJobs((previous) => {
          const { [jobId]: _, ...rest } = previous;
          return rest;
        });
        toast.error(failureError ?? "Server deployment failed");
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      launchedUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const fetchInstances = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [fetched, activeJobs] = await Promise.all([
        invokeCommand<Instance[]>("list_instances"),
        invokeCommand<ActiveSpawnJob[]>("list_active_spawn_jobs"),
      ]);

      const newSpawnJobs: Record<string, SpawnJobState> = {};
      const spawningPlaceholders: Instance[] = [];

      for (const activeJob of activeJobs) {
        const instanceId =
          activeJob.instanceId ?? `spawning-recovered-${activeJob.jobId}`;
        newSpawnJobs[activeJob.jobId] = {
          jobId: activeJob.jobId,
          instanceId,
          steps: activeJob.steps.map((step) => ({
            ...step,
            status: activeJob.stepStatuses[step.id] ?? SpawnStepStatus.Pending,
          })),
        };

        if (!activeJob.instanceId) {
          spawningPlaceholders.push({
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

      const sorted = [...fetched].sort((a, b) => {
        const installingA = a.state === InstanceState.Installing ? 0 : 1;
        const installingB = b.state === InstanceState.Installing ? 0 : 1;
        return installingA - installingB;
      });
      setInstances([...spawningPlaceholders, ...sorted]);
    } catch (fetchError) {
      const message =
        fetchError instanceof Error
          ? fetchError.message
          : "Failed to fetch instances";
      console.error("Failed to fetch instances:", fetchError);
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

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
          steps: job.steps.map((step, index) => ({
            ...step,
            status:
              index === 0 ? SpawnStepStatus.Running : SpawnStepStatus.Pending,
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

  return {
    instances,
    isLoading,
    isSpawning: Object.keys(spawnJobs).length > 0,
    terminatingInstanceId,
    error,
    spawnInstance,
    terminateInstance,
    clearError,
    refetch: fetchInstances,
    getSpawnJobForInstance,
  };
}
