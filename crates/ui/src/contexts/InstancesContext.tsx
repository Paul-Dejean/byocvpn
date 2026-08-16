import { createContext, useContext, ReactNode } from "react";
import { CloudProviderName, Instance, SpawnJob } from "../types";
import { useInstances } from "../hooks/useInstances";

interface InstancesContextValue {
  instances: Instance[];
  spawnJobs: SpawnJob[];
  pendingSpawnJobs: SpawnJob[];
  isLoading: boolean;
  isRefreshing: boolean;
  terminatingInstanceId: string | null;
  startSpawnJob: (
    region: string,
    provider: CloudProviderName,
  ) => Promise<SpawnJob | null>;
  terminateInstance: (
    instanceId: string,
    region: string,
    provider: CloudProviderName,
  ) => Promise<void>;
  dismissSpawnJob: (jobId: string) => Promise<void>;
  refetchInstances: () => Promise<void>;
}

const InstancesContext = createContext<InstancesContextValue | null>(null);

interface InstancesProviderProps {
  children: ReactNode;
}

export function InstancesProvider({ children }: InstancesProviderProps) {
  const {
    instances,
    spawnJobs,
    pendingSpawnJobs,
    isLoading,
    isRefreshing,
    terminatingInstanceId,
    startSpawnJob,
    dismissSpawnJob,
    terminateInstance,
    refetchInstances,
  } = useInstances();

  return (
    <InstancesContext.Provider
      value={{
        instances,
        spawnJobs,
        pendingSpawnJobs,
        isLoading,
        isRefreshing,
        terminatingInstanceId,
        startSpawnJob,
        terminateInstance,
        dismissSpawnJob,
        refetchInstances,
      }}
    >
      {children}
    </InstancesContext.Provider>
  );
}

export function useInstancesContext() {
  const context = useContext(InstancesContext);
  if (!context) {
    throw new Error(
      "useInstancesContext must be used within InstancesProvider",
    );
  }
  return context;
}
