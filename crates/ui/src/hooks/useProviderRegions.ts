import { useState, useEffect, useRef } from "react";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { listen } from "@tauri-apps/api/event";
import { invokeCommand } from "../lib/invokeCommand";
import toast from "react-hot-toast";
import {
  CloudProviderName,
  Region,
  RegionGroup,
  SpawnStep,
  SpawnStepState,
  SpawnStepStatus,
} from "../types";

interface EnableRegionJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: CloudProviderName;
}

interface EnableRegionProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
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
  steps: SpawnStepState[];
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

async function fetchEnabledRegions(
  provider: CloudProviderName,
  regions: Region[],
): Promise<Set<string>> {
  const store = await loadStore("providers.json");
  const enabled = new Set<string>();
  for (const region of regions) {
    const value = await store.get<boolean>(
      `enabled_regions/${provider}/${region.name}`,
    );
    if (value === true) enabled.add(region.name);
  }
  return enabled;
}

export function useProviderRegions(provider: CloudProviderName) {
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [enabledRegions, setEnabledRegions] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(true);
  const [activeEnableJob, setActiveEnableJob] =
    useState<EnableRegionJobState | null>(null);
  const [isEnableDrawerOpen, setIsEnableDrawerOpen] = useState(false);
  const [isEnableComplete, setIsEnableComplete] = useState(false);
  const [enableError, setEnableError] = useState<string | null>(null);
  const activeEnableJobIdRef = useRef<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setGroupedRegions([]);
    setEnabledRegions(new Set());

    invokeCommand<Region[]>("get_regions", { provider })
      .then(async (regions) => {
        setGroupedRegions(groupRegionsByCountry(regions));
        setEnabledRegions(await fetchEnabledRegions(provider, regions));
      })
      .catch(console.error)
      .finally(() => setIsLoading(false));
  }, [provider]);

  useEffect(() => {
    const progressUnlisten = listen<EnableRegionProgressEvent>(
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

    const completeUnlisten = listen<EnableRegionCompleteEvent>(
      "enable-region-complete",
      ({ payload }) => {
        if (activeEnableJobIdRef.current === payload.jobId) {
          setIsEnableComplete(true);
          setEnabledRegions((previous) => new Set([...previous, payload.region]));
          toast.success(`${payload.region} enabled!`);
        }
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      "enable-region-failed",
      ({ payload }) => {
        if (activeEnableJobIdRef.current === payload.jobId) {
          setEnableError(payload.error);
        }
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const enableRegion = async (region: Region) => {
    try {
      const job = await invokeCommand<EnableRegionJob>("enable_region", {
        region: region.name,
        provider,
      });
      const initialSteps: SpawnStepState[] = job.steps.map((step) => ({
        ...step,
        status: SpawnStepStatus.Pending,
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
    groupedRegions,
    enabledRegions,
    isLoading,
    enableRegion,
    activeEnableJob,
    isEnableDrawerOpen,
    isEnableComplete,
    enableError,
    closeEnableDrawer,
  };
}
