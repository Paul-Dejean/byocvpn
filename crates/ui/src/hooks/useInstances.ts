import { useEffect, useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import { CloudProviderName, Instance, InstanceState } from "../types";
import { invokeCommand } from "../lib/invokeCommand";
import { INSTANCES_QUERY_KEY } from "./useSpawnJobs";

const AUTO_TERMINATED_EVENT = "instance-auto-terminated";

function fetchInstances(): Promise<Instance[]> {
  return invokeCommand<Instance[]>("list_instances");
}

function compareByDeploymentProgress(first: Instance, second: Instance) {
  const firstRank = first.state === InstanceState.Installing ? 0 : 1;
  const secondRank = second.state === InstanceState.Installing ? 0 : 1;
  return firstRank - secondRank;
}

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
    queryFn: fetchInstances,
    staleTime: 0,
    refetchOnReconnect: false,
  });

  const instances = useMemo(
    () => [...fetchedInstances].sort(compareByDeploymentProgress),
    [fetchedInstances],
  );

  useEffect(() => {
    const autoTerminatedUnlisten = listen(AUTO_TERMINATED_EVENT, () => {
      queryClient.invalidateQueries({ queryKey: INSTANCES_QUERY_KEY });
    });

    return () => {
      autoTerminatedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  async function refetchInstances(): Promise<void> {
    await queryClient.invalidateQueries({ queryKey: INSTANCES_QUERY_KEY });
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
    } catch (terminateError) {
      const message =
        terminateError instanceof Error
          ? terminateError.message
          : "Failed to terminate instance";
      toast.error(message);
      throw terminateError;
    } finally {
      setTerminatingInstanceId(null);
    }
  }

  return {
    instances,
    isLoading,
    isRefreshing: isFetching && !isLoading,
    terminatingInstanceId,
    terminateInstance,
    refetchInstances,
  };
}
