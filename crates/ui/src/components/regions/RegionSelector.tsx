import { useInstancesContext } from "../../contexts";
import { getRegionInfo } from "../../types/regionInfo";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { load as loadStore } from "@tauri-apps/plugin-store";
import toast from "react-hot-toast";
import {
  Instance,
  EnableRegionJob,
  EnableRegionProgressEvent,
  EnableRegionCompleteEvent,
  SpawnStepState,
} from "../../types";
import { ProvisionAccountDrawer } from "../settings/ProvisionAccountDrawer";

interface SimpleRegion {
  name: string;
  country: string;
}

interface SimpleRegionGroup {
  continent: string;
  regions: SimpleRegion[];
}

interface RegionSelectorProps {
  provider: string;
  onClose: () => void;
  onSpawned?: (instance: Instance) => void;
}

export function RegionSelector({
  provider,
  onClose,
  onSpawned,
}: RegionSelectorProps) {
  const [selectedRegion, setSelectedRegion] = useState<SimpleRegion | null>(null);
  const [groupedRegions, setGroupedRegions] = useState<SimpleRegionGroup[]>([]);
  const [isLoadingRegions, setIsLoadingRegions] = useState(true);
  const [enabledRegions, setEnabledRegions] = useState<Set<string>>(new Set());

  const [activeEnableJob, setActiveEnableJob] = useState<{
    jobId: string;
    region: string;
    steps: SpawnStepState[];
  } | null>(null);
  const [isEnableDrawerOpen, setIsEnableDrawerOpen] = useState(false);
  const [isEnableComplete, setIsEnableComplete] = useState(false);
  const [enableError, setEnableError] = useState<string | null>(null);

  const activeEnableJobIdRef = useRef<string | null>(null);

  const { spawnInstance, instances } = useInstancesContext();

  useEffect(() => {
    setIsLoadingRegions(true);
    setSelectedRegion(null);
    invoke<SimpleRegion[]>("get_regions", { provider })
      .then(async (regions) => {
        const groups: Record<string, SimpleRegion[]> = {};
        regions.forEach((region) => {
          if (!groups[region.country]) groups[region.country] = [];
          groups[region.country].push(region);
        });
        setGroupedRegions(
          Object.entries(groups)
            .map(([continent, continentRegions]) => ({
              continent,
              regions: continentRegions.sort((a, b) =>
                a.name.localeCompare(b.name),
              ),
            }))
            .sort((a, b) => a.continent.localeCompare(b.continent)),
        );

        const store = await loadStore("providers.json");
        const enabled = new Set<string>();
        for (const region of regions) {
          const value = await store.get<boolean>(
            `enabled_regions/${provider}/${region.name}`,
          );
          if (value === true) {
            enabled.add(region.name);
          }
        }
        setEnabledRegions(enabled);
      })
      .catch(console.error)
      .finally(() => setIsLoadingRegions(false));
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

  const handleEnableRegion = async (
    regionName: string,
    event: React.MouseEvent,
  ) => {
    event.stopPropagation();
    try {
      const job = await invoke<EnableRegionJob>("enable_region", {
        region: regionName,
        provider,
      });
      const initialSteps: SpawnStepState[] = job.steps.map((step) => ({
        ...step,
        status: "pending" as const,
      }));
      activeEnableJobIdRef.current = job.jobId;
      setActiveEnableJob({
        jobId: job.jobId,
        region: regionName,
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

  const lookupRegionInfo = (regionName: string) => getRegionInfo(provider, regionName);

  const handleDeploy = async () => {
    if (selectedRegion && enabledRegions.has(selectedRegion.name)) {
      const placeholder = await spawnInstance(selectedRegion.name, provider);
      onSpawned?.(placeholder);
      onClose();
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900">
      <div className="bg-gray-800 border-b border-gray-700/50 p-6">
        <div className="flex items-center gap-4">
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition"
          >
            <svg
              className="w-6 h-6 text-gray-300"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 19l-7-7 7-7"
              />
            </svg>
          </button>
          <div>
            <h1 className="text-3xl font-bold text-blue-400">Deploy New Server</h1>
            <p className="text-gray-300 mt-1">
              Enable a region first, then deploy your VPN server
            </p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 pb-0">
        {isLoadingRegions ? (
          <div className="flex justify-center items-center h-32">
            <div className="w-8 h-8 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
          </div>
        ) : (
          <div className="space-y-6 pb-6">
            {groupedRegions.map((group, idx) => (
              <div key={idx}>
                <h3 className="text-xs uppercase text-gray-400 font-semibold mb-3 px-2">
                  {group.continent}
                </h3>
                <div className="grid grid-cols-2 gap-3">
                  {group.regions.map((region) => {
                    const isEnabled = enabledRegions.has(region.name);
                    const isSelected = selectedRegion?.name === region.name;
                    const activeInstanceCount = instances.filter(
                      (instance) => instance.region === region.name,
                    ).length;

                    return (
                      <div
                        key={region.name}
                        onClick={() => isEnabled && setSelectedRegion(region)}
                        className={`p-4 border rounded-lg transition-all ${
                          isEnabled
                            ? isSelected
                              ? "bg-blue-900/30 border-blue-500 cursor-pointer"
                              : "bg-gray-800 border-gray-700 hover:border-gray-600 hover:bg-gray-700 cursor-pointer"
                            : "bg-gray-800/40 border-gray-700/40 cursor-default"
                        }`}
                      >
                        <div className="flex items-start gap-3 mb-3">
                          <span
                            className={`text-2xl leading-none mt-0.5 ${!isEnabled ? "opacity-30" : ""}`}
                          >
                            {lookupRegionInfo(region.name).flag}
                          </span>
                          <div className="flex-1 min-w-0">
                            <p
                              className={`font-medium text-sm ${isEnabled ? "text-white" : "text-gray-600"}`}
                            >
                              {lookupRegionInfo(region.name).city}
                            </p>
                            <p
                              className={`text-xs mt-0.5 ${isEnabled ? "text-gray-400" : "text-gray-700"}`}
                            >
                              {region.name}
                            </p>
                          </div>
                          {isSelected && (
                            <div className="w-3 h-3 bg-blue-400 rounded-full flex-shrink-0 mt-1" />
                          )}
                        </div>

                        <div className="flex items-center justify-between">
                          {activeInstanceCount > 0 && isEnabled && (
                            <div className="flex items-center gap-1 text-xs text-gray-400">
                              <svg
                                className="w-3 h-3"
                                fill="currentColor"
                                viewBox="0 0 20 20"
                              >
                                <path
                                  fillRule="evenodd"
                                  d="M2 5a2 2 0 012-2h12a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm3.293 1.293a1 1 0 011.414 0l3 3a1 1 0 010 1.414l-3 3a1 1 0 01-1.414-1.414L7.586 10 5.293 7.707a1 1 0 010-1.414zM11 12a1 1 0 100 2h3a1 1 0 100-2h-3z"
                                  clipRule="evenodd"
                                />
                              </svg>
                              <span>{activeInstanceCount} active</span>
                            </div>
                          )}
                          {isEnabled ? (
                            <div className="flex items-center gap-1 text-xs text-green-500 ml-auto">
                              <svg
                                className="w-3 h-3"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                              >
                                <path
                                  strokeLinecap="round"
                                  strokeLinejoin="round"
                                  strokeWidth={2}
                                  d="M5 13l4 4L19 7"
                                />
                              </svg>
                              <span>Enabled</span>
                            </div>
                          ) : (
                            <button
                              onClick={(e) => handleEnableRegion(region.name, e)}
                              className="ml-auto text-xs px-2.5 py-1 bg-gray-700 hover:bg-gray-600 text-gray-400 hover:text-white border border-gray-600 rounded transition flex items-center gap-1.5"
                            >
                              <svg
                                className="w-3 h-3"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                              >
                                <path
                                  strokeLinecap="round"
                                  strokeLinejoin="round"
                                  strokeWidth={2}
                                  d="M13 10V3L4 14h7v7l9-11h-7z"
                                />
                              </svg>
                              Enable
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {selectedRegion && enabledRegions.has(selectedRegion.name) && (
        <div className="border-t border-gray-700 p-6 flex-shrink-0">
          <button
            onClick={handleDeploy}
            className="w-full px-6 py-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition font-medium text-lg shadow-lg flex items-center justify-center gap-2"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
            Deploy Server in {lookupRegionInfo(selectedRegion.name).city}
          </button>
        </div>
      )}

      <ProvisionAccountDrawer
        isOpen={isEnableDrawerOpen}
        onClose={() => setIsEnableDrawerOpen(false)}
        provider={provider}
        title={activeEnableJob ? `Enabling ${activeEnableJob.region}` : ""}
        subtitle="Setting up regional infrastructure"
        successMessage={
          activeEnableJob
            ? `${activeEnableJob.region} is ready for deployment`
            : undefined
        }
        steps={activeEnableJob?.steps ?? []}
        isComplete={isEnableComplete}
        error={enableError}
      />
    </div>
  );
}
