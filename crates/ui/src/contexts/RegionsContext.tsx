import { createContext, useContext, ReactNode } from "react";
import { AwsRegion, RegionGroup } from "../types";
import { useRegions } from "../hooks/useRegions";

interface RegionsContextValue {
  regions: AwsRegion[];
  groupedRegions: RegionGroup[];
  isLoading: boolean;
  error: string | null;
  loadRegions: () => Promise<void>;
  clearError: () => void;
}

const RegionsContext = createContext<RegionsContextValue | null>(null);

interface RegionsProviderProps {
  children: ReactNode;
}

export function RegionsProvider({ children }: RegionsProviderProps) {
  const regionsState = useRegions();

  return (
    <RegionsContext.Provider value={regionsState}>
      {children}
    </RegionsContext.Provider>
  );
}

export function useRegionsContext() {
  const context = useContext(RegionsContext);
  if (!context) {
    throw new Error("useRegionsContext must be used within RegionsProvider");
  }
  return context;
}
