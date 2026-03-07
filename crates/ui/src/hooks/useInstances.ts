import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  AwsRegion,
  Instance,
  SpawnJob,
  SpawnJobState,
  SpawnProgressEvent,
  SpawnCompleteEvent,
} from "../types";

/**
 * Hook for managing cloud instances — listing, spawning, and terminating.
 *
 * Spawn is fully async: `spawnInstance` returns as soon as the backend
 * acknowledges the job (with a `SpawnJob` describing the steps). Progress is
 * pushed back via Tauri events and reflected in per-job step state returned
 * by `getSpawnJobForInstance`.
 */
export const useInstances = (regions: AwsRegion[]) => {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [terminatingInstanceId, setTerminatingInstanceId] = useState<
    string | null
  >(null);
  const [error, setError] = useState<string | null>(null);

  /** Live step states keyed by jobId. */
  const [spawnJobs, setSpawnJobs] = useState<Record<string, SpawnJobState>>({});

  /**
   * Maps a spawning placeholder instance id (e.g. "spawning-1234") to its
   * backend jobId so we can correlate events with the right list entry.
   */
  const tempToJob = useRef<Record<string, string>>({});

  useEffect(() => {
    if (regions.length > 0) fetchInstances();
  }, [regions]);

  // Register spawn event listeners once on mount and clean up on unmount.
  useEffect(() => {
    const progressUnlisten = listen<SpawnProgressEvent>(
      "spawn-progress",
      ({ payload }) => {
        const { jobId, stepId, status, error } = payload;
        setSpawnJobs((prev) => {
          const job = prev[jobId];
          if (!job) return prev;
          return {
            ...prev,
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

    const completeUnlisten = listen<SpawnCompleteEvent>(
      "spawn-complete",
      ({ payload }) => {
        const { jobId, instance } = payload;
        const tempId = Object.entries(tempToJob.current).find(
          ([, jid]) => jid === jobId,
        )?.[0];
        if (tempId) {
          setInstances((prev) =>
            prev.map((inst) => (inst.id === tempId ? instance : inst)),
          );
          delete tempToJob.current[tempId];
        }
        setSpawnJobs((prev) => {
          const { [jobId]: _, ...rest } = prev;
          return rest;
        });
        toast.success("Server deployed successfully!");
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      "spawn-failed",
      ({ payload }) => {
        const { jobId, error } = payload;
        const tempId = Object.entries(tempToJob.current).find(
          ([, jid]) => jid === jobId,
        )?.[0];
        if (tempId) {
          setInstances((prev) => prev.filter((inst) => inst.id !== tempId));
          delete tempToJob.current[tempId];
        }
        setSpawnJobs((prev) => {
          const { [jobId]: _, ...rest } = prev;
          return rest;
        });
        toast.error(error ?? "Server deployment failed");
      },
    );

    return () => {
      progressUnlisten.then((u) => u());
      completeUnlisten.then((u) => u());
      failedUnlisten.then((u) => u());
    };
  }, []);

  const fetchInstances = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const fetched = await invoke<Instance[]>("list_instances");
      setInstances(fetched);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to fetch instances";
      console.error("Failed to fetch existing instances:", err);
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Begin deploying a new instance.  Returns as soon as the backend has
   * accepted the job — the actual deployment runs in the background and
   * updates arrive via `spawn-progress` / `spawn-complete` / `spawn-failed`.
   */
  const spawnInstance = async (
    regionName: string,
    provider: string,
  ): Promise<Instance> => {
    const tempId = `spawning-${Date.now()}`;

    const placeholder: Instance = {
      id: tempId,
      name: "Deploying…",
      state: "spawning",
      publicIpV4: "",
      publicIpV6: "",
      region: regionName,
      provider,
    };

    // Add a placeholder card immediately so the UI shows something right away.
    setInstances((prev) => [...prev, placeholder]);

    try {
      const job = await invoke<SpawnJob>("spawn_instance", {
        region: regionName,
        provider,
      });

      // Associate the placeholder with the backend job.
      tempToJob.current[tempId] = job.jobId;

      // Initialise all steps as pending so the UI can render the full list.
      setSpawnJobs((prev) => ({
        ...prev,
        [job.jobId]: {
          jobId: job.jobId,
          steps: job.steps.map((step) => ({
            ...step,
            status: "pending" as const,
          })),
        },
      }));
    } catch (err) {
      // If the invoke itself failed (e.g. bad credentials), remove placeholder.
      setInstances((prev) => prev.filter((inst) => inst.id !== tempId));
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
    provider: string,
  ): Promise<void> => {
    setTerminatingInstanceId(instanceId);
    setError(null);
    try {
      await invoke("terminate_instance", { instanceId, region, provider });
      setInstances((prev) => prev.filter((i) => i.id !== instanceId));
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

  /**
   * Returns the live spawn job state (step list + statuses) for the given
   * instance placeholder id, or `undefined` if the instance is not spawning.
   */
  const getSpawnJobForInstance = (
    instanceId: string,
  ): SpawnJobState | undefined => {
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
};
