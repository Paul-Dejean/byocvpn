import { QueryClient } from "@tanstack/react-query";
import { Channel } from "@tauri-apps/api/core";
import { EnableRegionEvent, commands } from "../bindings";
import { providerRegionsQueryOptions } from "../queries/providerRegions";
import { CloudProviderName, JobStepStatus, Region } from "../types";

const EnableRegionEventKind = {
  Started: "STARTED",
  Progress: "PROGRESS",
  Complete: "COMPLETE",
  Failed: "FAILED",
} as const satisfies Record<string, EnableRegionEvent["kind"]>;

export type EnableRegionProgressListener = (status: JobStepStatus) => void;

export async function isRegionEnabled(
  queryClient: QueryClient,
  provider: CloudProviderName,
  regionName: string,
): Promise<boolean> {
  const regionsData = await queryClient.ensureQueryData(
    providerRegionsQueryOptions(provider),
  );
  return regionsData.enabledRegions.has(regionName);
}

export function enableRegion(
  queryClient: QueryClient,
  provider: CloudProviderName,
  region: Region,
  onProgress: EnableRegionProgressListener,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const onEvent = new Channel<EnableRegionEvent>();
    onEvent.onmessage = (event) => {
      switch (event.kind) {
        case EnableRegionEventKind.Started:
          onProgress(JobStepStatus.Running);
          return;
        case EnableRegionEventKind.Progress:
          if (event.status === JobStepStatus.Failed) {
            onProgress(JobStepStatus.Failed);
          }
          return;
        case EnableRegionEventKind.Complete:
          markRegionEnabled(queryClient, provider, event.region);
          onProgress(JobStepStatus.Completed);
          resolve();
          return;
        case EnableRegionEventKind.Failed:
          onProgress(JobStepStatus.Failed);
          reject(new Error(event.error));
          return;
      }
    };

    commands.enableRegion(region.name, provider, onEvent).then((result) => {
      if (result.status === "error") {
        onProgress(JobStepStatus.Failed);
        reject(new Error(result.error));
      }
    });
  });
}

function markRegionEnabled(
  queryClient: QueryClient,
  provider: CloudProviderName,
  regionName: string,
) {
  queryClient.setQueryData(
    providerRegionsQueryOptions(provider).queryKey,
    (previous) =>
      previous
        ? {
            ...previous,
            enabledRegions: new Set([...previous.enabledRegions, regionName]),
          }
        : previous,
  );
}
