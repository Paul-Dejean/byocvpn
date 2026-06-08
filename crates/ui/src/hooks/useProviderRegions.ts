import { useState, useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { listen } from "@tauri-apps/api/event";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";
import {
  CloudProviderName,
  Region,
  RegionGroup,
  JobStep,
  JobStepState,
  JobStepStatus,
} from "../types";

interface EnableRegionJob {
  jobId: string;
  steps: JobStep[];
  region: string;
  provider: CloudProviderName;
}

interface EnableRegionProgressEvent {
  jobId: string;
  stepId: string;
  status: JobStepStatus;
  error?: string;
}

interface EnableRegionCompleteEvent {
  jobId: string;
  region: string;
  provider: CloudProviderName;
}

export interface EnableRegionJobState {
  jobId: string;
  region: string;
  country: string;
  steps: JobStepState[];
}

interface ProviderRegionsData {
  groupedRegions: RegionGroup[];
  enabledRegions: Set<string>;
}

function groupRegionsByCountry(regions: Region[]): RegionGroup[] {
  const groups: Record<string, Region[]> = {};
  for (const region of regions) {
    if (!groups[region.country]) groups[region.country] = [];
    groups[region.country].push(region);
  }
  return Object.entries(groups)
    .map(([continent, continentRegions]) => ({
      continent,
      regions: continentRegions.sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .sort((a, b) => a.continent.localeCompare(b.continent));
}

async function fetchProviderRegions(
  provider: CloudProviderName,
): Promise<ProviderRegionsData> {
  const regions = await invokeCommand<Region[]>("get_regions", { provider });
  const store = await loadStore("providers.json");
  const enabledRegions = new Set<string>();
  for (const region of regions) {
    const value = await store.get<boolean>(
      `enabled_regions/${provider}/${region.name}`,
    );
    if (value === true) enabledRegions.add(region.name);
  }
  return { groupedRegions: groupRegionsByCountry(regions), enabledRegions };
}

export function useProviderRegions(provider: CloudProviderName) {
  const queryClient = useQueryClient();
  const [activeEnableJob, setActiveEnableJob] =
    useState<EnableRegionJobState | null>(null);
  const [isEnableDrawerOpen, setIsEnableDrawerOpen] = useState(false);
  const [isEnableComplete, setIsEnableComplete] = useState(false);
  const [enableError, setEnableError] = useState<string | null>(null);
  const activeEnableJobIdRef = useRef<string | null>(null);

  const { data, isLoading, isFetching } = useQuery({
    queryKey: ["regions", provider],
    queryFn: () => fetchProviderRegions(provider),
    staleTime: 30_000,
  });

  useEffect(() => {
    let canceled = false;
    const registeredUnlisteners: Array<() => void> = [];

    const registerListeners = async () => {
      const unlistenProgress = await listen<EnableRegionProgressEvent>(
        "enable-region-progress",
        ({ payload }) => {
          const { jobId, stepId, status, error } = payload;
          setActiveEnableJob((previous) => {
            if (!previous || previous.jobId !== jobId) return previous;
            return {
              ...previous,
              steps: previous.steps.map((step) =>
                step.id === stepId ? { ...step, status, error } : step,
              ),
            };
          });
        },
      );

      const unlistenComplete = await listen<EnableRegionCompleteEvent>(
        "enable-region-complete",
        ({ payload }) => {
          if (activeEnableJobIdRef.current !== payload.jobId) return;
          setIsEnableComplete(true);
          queryClient.setQueryData<ProviderRegionsData>(
            ["regions", provider],
            (previous) => {
              if (!previous) return previous;
              const enabledRegions = new Set([
                ...previous.enabledRegions,
                payload.region,
              ]);
              return { ...previous, enabledRegions };
            },
          );
          toast.success(`${payload.region} enabled!`);
        },
      );

      const unlistenFailed = await listen<{ jobId: string; error: string }>(
        "enable-region-failed",
        ({ payload }) => {
          if (activeEnableJobIdRef.current === payload.jobId) {
            setEnableError(payload.error);
          }
        },
      );

      if (canceled) {
        unlistenProgress();
        unlistenComplete();
        unlistenFailed();
      } else {
        registeredUnlisteners.push(
          unlistenProgress,
          unlistenComplete,
          unlistenFailed,
        );
      }
    };

    registerListeners();

    return () => {
      canceled = true;
      registeredUnlisteners.forEach((unlisten) => unlisten());
    };
  }, [provider]);

  const enableRegion = async (region: Region) => {
    try {
      const job = await invokeCommand<EnableRegionJob>("enable_region", {
        region: region.name,
        provider,
      });
      const initialSteps: JobStepState[] = job.steps.map((step, index) => ({
        ...step,
        status: index === 0 ? JobStepStatus.Running : JobStepStatus.Pending,
      }));
      activeEnableJobIdRef.current = job.jobId;
      setActiveEnableJob({
        jobId: job.jobId,
        region: region.name,
        country: region.country,
        steps: initialSteps,
      });
      setIsEnableComplete(false);
      setEnableError(null);
      setIsEnableDrawerOpen(true);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to enable region";
      toast.error(message);
    }
  };

  const closeEnableDrawer = () => setIsEnableDrawerOpen(false);

  return {
    groupedRegions: data?.groupedRegions ?? [],
    enabledRegions: data?.enabledRegions ?? new Set<string>(),
    isLoading,
    isRefetching: isFetching && !isLoading,
    enableRegion,
    activeEnableJob,
    isEnableDrawerOpen,
    isEnableComplete,
    enableError,
    closeEnableDrawer,
  };
}
