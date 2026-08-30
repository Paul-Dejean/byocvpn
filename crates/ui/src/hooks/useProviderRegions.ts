import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import { EnableRegionEvent, commands } from "../bindings";
import { providerRegionsQueryOptions } from "../queries/providerRegions";
import toast from "react-hot-toast";
import {
  CloudProviderName,
  Region,
  JobStepState,
  JobStepStatus,
} from "../types";

const EnableRegionEventKind = {
  Started: "STARTED",
  Progress: "PROGRESS",
  Complete: "COMPLETE",
  Failed: "FAILED",
} as const satisfies Record<string, EnableRegionEvent["kind"]>;

export interface EnableRegionJobState {
  jobId: string;
  region: string;
  country: string;
  steps: JobStepState[];
}

export function useProviderRegions(provider: CloudProviderName) {
  const queryClient = useQueryClient();
  const [activeEnableJob, setActiveEnableJob] =
    useState<EnableRegionJobState | null>(null);
  const [isEnableDrawerOpen, setIsEnableDrawerOpen] = useState(false);
  const [isEnableComplete, setIsEnableComplete] = useState(false);
  const [enableError, setEnableError] = useState<string | null>(null);

  const { data, isLoading, isFetching } = useQuery(
    providerRegionsQueryOptions(provider),
  );

  async function enableRegion(region: Region) {
    const onEvent = new Channel<EnableRegionEvent>();
    onEvent.onmessage = (event) => applyEnableRegionEvent(event, region);

    setActiveEnableJob(null);
    setIsEnableComplete(false);
    setEnableError(null);

    const result = await commands.enableRegion(region.name, provider, onEvent);
    if (result.status === "error") {
      toast.error(result.error);
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
            error: null,
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
        queryClient.setQueryData(
          providerRegionsQueryOptions(provider).queryKey,
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
