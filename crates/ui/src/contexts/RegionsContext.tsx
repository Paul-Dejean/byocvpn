import { createContext, useContext, ReactNode } from "react";
import { AwsRegion, RegionGroup } from "../types";
import { useRegions } from "../hooks/useRegions";

/**
 * Context value for regions management
 */
interface RegionsContextValue {
  regions: AwsRegion[];
  groupedRegions: RegionGroup[];
  isLoading: boolean;
  error: string | null;
  loadRegions: () => Promise<void>;
  clearError: () => void;
}

const RegionsContext = createContext<RegionsContextValue | null>(null);

/**
 * Props for RegionsProvider
 */
interface RegionsProviderProps {
  children: ReactNode;
}

/**
 * Provider component that manages regions state globally
 */
export function RegionsProvider({ children }: RegionsProviderProps) {
  const regionsState = useRegions();

  return (
    <RegionsContext.Provider value={regionsState}>
      {children}
    </RegionsContext.Provider>
  );
}

/**
 * Hook to access regions context
 * @throws Error if used outside RegionsProvider
 */
export function useRegionsContext() {
  const context = useContext(RegionsContext);
  if (!context) {
    throw new Error("useRegionsContext must be used within RegionsProvider");
  }
  return context;
}
