import { createContext, useContext, ReactNode } from "react";
import { Instance } from "../types";
import { useInstances } from "../hooks/useInstances";
import { useRegionsContext } from "./RegionsContext";

/**
 * Context value for instances management
 */
interface InstancesContextValue {
  instances: Instance[];
  isLoading: boolean;
  isSpawning: boolean;
  isTerminating: boolean;
  error: string | null;
  spawnInstance: (regionName: string) => Promise<Instance>;
  terminateInstance: (instanceId: string, region: string) => Promise<void>;
  clearError: () => void;
  refetch: () => Promise<void>;
}

const InstancesContext = createContext<InstancesContextValue | null>(null);

/**
 * Props for InstancesProvider
 */
interface InstancesProviderProps {
  children: ReactNode;
}

/**
 * Provider component that manages instances state globally
 */
export function InstancesProvider({ children }: InstancesProviderProps) {
  const { regions } = useRegionsContext();
  const instancesState = useInstances(regions);

  return (
    <InstancesContext.Provider value={instancesState}>
      {children}
    </InstancesContext.Provider>
  );
}

/**
 * Hook to access instances context
 * @throws Error if used outside InstancesProvider
 */
export function useInstancesContext() {
  const context = useContext(InstancesContext);
  if (!context) {
    throw new Error(
      "useInstancesContext must be used within InstancesProvider"
    );
  }
  return context;
}
