import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { load as loadStore } from "@tauri-apps/plugin-store";
import { Channel } from "@tauri-apps/api/core";
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

enum EnableRegionEventKind {
  Started = "STARTED",
  Progress = "PROGRESS",
  Complete = "COMPLETE",
  Failed = "FAILED",
}

type EnableRegionEvent =
  | { kind: EnableRegionEventKind.Started; jobId: string; steps: JobStep[] }
  | {
      kind: EnableRegionEventKind.Progress;
      stepId: string;
      status: JobStepStatus;
      error?: string;
    }
  | { kind: EnableRegionEventKind.Complete; region: string }
  | { kind: EnableRegionEventKind.Failed; error: string };

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

  const { data, isLoading, isFetching } = useQuery({
    queryKey: ["regions", provider],
    queryFn: () => fetchProviderRegions(provider),
    staleTime: 30_000,
  });

  async function enableRegion(region: Region) {
    const onEvent = new Channel<EnableRegionEvent>();
    onEvent.onmessage = (event) => applyEnableRegionEvent(event, region);

    setActiveEnableJob(null);
    setIsEnableComplete(false);
    setEnableError(null);

    try {
      await invokeCommand("enable_region", {
        region: region.name,
        provider,
        onEvent,
      });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to enable region";
      toast.error(message);
    }
  }

  function applyEnableRegionEvent(event: EnableRegionEvent, region: Region) {
    switch (event.kind) {
      case EnableRegionEventKind.Started:
        setActiveEnableJob({
          jobId: event.jobId,
          region: region.name,
          country: region.country,
          steps: event.steps.map((step) => ({
            ...step,
            status: JobStepStatus.Pending,
          })),
        });
        setIsEnableDrawerOpen(true);
        return;

      case EnableRegionEventKind.Progress:
        setActiveEnableJob((previous) =>
          previous
            ? {
                ...previous,
                steps: previous.steps.map((step) =>
                  step.id === event.stepId
                    ? { ...step, status: event.status, error: event.error }
                    : step,
                ),
              }
            : previous,
        );
        return;

      case EnableRegionEventKind.Complete:
        setIsEnableComplete(true);
        queryClient.setQueryData<ProviderRegionsData>(
          ["regions", provider],
          (previous) =>
            previous
              ? {
                  ...previous,
                  enabledRegions: new Set([
                    ...previous.enabledRegions,
                    event.region,
                  ]),
                }
              : previous,
        );
        return;

      case EnableRegionEventKind.Failed:
        setEnableError(event.error);
        return;
    }
  }

  function closeEnableDrawer() {
    setIsEnableDrawerOpen(false);
  }

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
