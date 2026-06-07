import { createContext, useContext, ReactNode } from "react";
import { CloudProviderName, Instance, SpawnJobState } from "../types";
import { useInstances } from "../hooks/useInstances";
import { useRegionsContext } from "./RegionsContext";

interface InstancesContextValue {
  instances: Instance[];
  isLoading: boolean;
  isSpawning: boolean;
  terminatingInstanceId: string | null;
  error: string | null;
  spawnInstance: (regionName: string, provider: CloudProviderName) => Promise<Instance>;
  terminateInstance: (
    instanceId: string,
    region: string,
    provider: CloudProviderName,
  ) => Promise<void>;
  clearError: () => void;
  dismissFailedInstance: (instanceId: string) => void;
  refetch: () => Promise<void>;
  getSpawnJobForInstance: (instanceId: string) => SpawnJobState | undefined;
}

const InstancesContext = createContext<InstancesContextValue | null>(null);

interface InstancesProviderProps {
  children: ReactNode;
}

export function InstancesProvider({ children }: InstancesProviderProps) {
  const { regions } = useRegionsContext();
  const instancesState = useInstances(regions);

  return (
    <InstancesContext.Provider value={instancesState}>
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
