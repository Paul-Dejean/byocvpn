import { useState, useEffect, useRef } from "react";
import { invokeCommand } from "../lib/invokeCommand";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  ActiveSpawnJob,
  Region,
  CloudProviderName,
  Instance,
  InstanceState,
  SpawnCompleteEvent,
  SpawnInstanceLaunchedEvent,
  SpawnJob,
  SpawnJobState,
  SpawnProgressEvent,
  SpawnStepStatus,
} from "../types";

export function useInstances(regions: Region[]) {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [spawnJobs, setSpawnJobs] = useState<Record<string, SpawnJobState>>({});

  const tempToJob = useRef<Record<string, string>>({});
  const jobToTemp = useRef<Record<string, string>>({});

  const trackJob = (tempId: string, jobId: string) => {
    tempToJob.current[tempId] = jobId;
    jobToTemp.current[jobId] = tempId;
  };

  const untrackJob = (jobId: string) => {
    const tempId = jobToTemp.current[jobId];
    if (tempId) {
      delete tempToJob.current[tempId];
      delete jobToTemp.current[jobId];
    }
  };

  useEffect(() => {
    if (regions.length > 0) fetchInstances();
  }, [regions]);

  useEffect(() => {
    const progressUnlisten = listen<SpawnProgressEvent>(
      "spawn-progress",
      ({ payload }) => {
        const { jobId, stepId, status, error } = payload;
        setSpawnJobs((previous) => {
          const job = previous[jobId];
          if (!job) return previous;
          return {
            ...previous,
            [jobId]: {
              ...job,
              steps: job.steps.map((step) =>
                step.id === stepId ? { ...step, status, error } : step,
              ),
            },
          };
        });
      },
    );

    const launchedUnlisten = listen<SpawnInstanceLaunchedEvent>(
      "spawn-instance-launched",
      ({ payload }) => {
        const { jobId, instance } = payload;
        const tempId = jobToTemp.current[jobId];
        if (tempId) {
          setInstances((previous) =>
            previous.map((existing) => (existing.id === tempId ? instance : existing)),
          );
          untrackJob(jobId);
          trackJob(instance.id, jobId);
        }
      },
    );

    const completeUnlisten = listen<SpawnCompleteEvent>(
      "spawn-complete",
      ({ payload }) => {
        const { jobId, instance } = payload;
        const tempId = jobToTemp.current[jobId];
        if (tempId) {
          setInstances((previous) =>
            previous.map((existing) => (existing.id === tempId ? instance : existing)),
          );
          untrackJob(jobId);
        }
        setSpawnJobs((previous) => {
          const { [jobId]: _, ...rest } = previous;
          return rest;
        });
        toast.success("Server deployed successfully!");
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      "spawn-failed",
      ({ payload }) => {
        const { jobId, error } = payload;
        const tempId = jobToTemp.current[jobId];
        if (tempId) {
          setInstances((previous) =>
            previous.filter((existing) => existing.id !== tempId),
          );
          untrackJob(jobId);
        }
        setSpawnJobs((previous) => {
          const { [jobId]: _, ...rest } = previous;
          return rest;
        });
        toast.error(error ?? "Server deployment failed");
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
        newSpawnJobs[activeJob.jobId] = {
          jobId: activeJob.jobId,
          steps: activeJob.steps.map((step) => ({
            ...step,
            status: activeJob.stepStatuses[step.id] ?? SpawnStepStatus.Pending,
          })),
        };

        if (activeJob.instanceId) {
          trackJob(activeJob.instanceId, activeJob.jobId);
        } else {
          const tempId = `spawning-recovered-${activeJob.jobId}`;
          trackJob(tempId, activeJob.jobId);
          spawningPlaceholders.push({
            id: tempId,
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
        const aInstalling = a.state === InstanceState.Installing ? 0 : 1;
        const bInstalling = b.state === InstanceState.Installing ? 0 : 1;
        return aInstalling - bInstalling;
      });
      setInstances([...spawningPlaceholders, ...sorted]);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to fetch instances";
      console.error("Failed to fetch instances:", err);
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

      trackJob(tempId, job.jobId);

      setSpawnJobs((previous) => ({
        ...previous,
        [job.jobId]: {
          jobId: job.jobId,
          steps: job.steps.map((step, index) => ({
            ...step,
            status: index === 0 ? SpawnStepStatus.Running : SpawnStepStatus.Pending,
          })),
        },
      }));
    } catch (err) {
      setInstances((previous) =>
        previous.filter((instance) => instance.id !== tempId),
      );
      const message =
        err instanceof Error ? err.message : "Failed to start deployment";
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
      await invokeCommand("terminate_instance", { instanceId, region, provider });
      setInstances((previous) =>
        previous.filter((instance) => instance.id !== instanceId),
      );
      toast.success("Server terminated successfully!");
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to terminate instance";
      setError(message);
      toast.error(message);
      console.error("Failed to terminate instance:", err);
      throw err;
    } finally {
      setTerminatingInstanceId(null);
    }
  };

  const getSpawnJobForInstance = (instanceId: string): SpawnJobState | undefined => {
    const jobId = tempToJob.current[instanceId];
    return jobId ? spawnJobs[jobId] : undefined;
  };

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
