import { Spinner } from "../common/Spinner";
import { useInstancesContext } from "../../contexts";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { useEffect, useState } from "react";
import { Instance, CloudProviderName, Region } from "../../types";
import { useProviderRegions } from "../../hooks/useProviderRegions";
import { ProvisionAccountDrawer } from "../settings/ProvisionAccountDrawer";

interface RegionSelectorProps {
  provider: CloudProviderName;
  onClose: () => void;
  onSpawned?: (instance: Instance) => void;
}

export function RegionSelector({
  provider,
  onClose,
  onSpawned,
}: RegionSelectorProps) {
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);
  const {
    groupedRegions,
    enabledRegions,
    isLoading: isLoadingRegions,
    enableRegion,
    activeEnableJob,
    isEnableDrawerOpen,
    isEnableComplete,
    enableError,
    closeEnableDrawer,
  } = useProviderRegions(provider);

  useEffect(() => {
    setSelectedRegion(null);
  }, [provider]);

  const { spawnInstance, instances } = useInstancesContext();

  const handleEnableRegion = async (region: Region, event: React.MouseEvent) => {
    event.stopPropagation();
    await enableRegion(region);
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
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
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
            <Spinner size="w-8 h-8" color="border-blue-400" />
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
                          <FlagIcon
                            countryCode={lookupRegionInfo(region.name).countryCode}
                            className={`text-2xl mt-0.5 ${!isEnabled ? "opacity-30" : ""}`}
                          />
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
                              onClick={(e) => handleEnableRegion(region, e)}
                              className="ml-auto text-xs px-2.5 py-1 bg-gray-700 hover:bg-gray-600 text-gray-400 hover:text-white border border-gray-600 rounded-lg transition-colors flex items-center gap-1.5"
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
            className="btn-primary w-full px-6 py-4 text-lg flex items-center justify-center gap-2"
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
        onClose={closeEnableDrawer}
        provider={provider}
        title={activeEnableJob ? `Enabling ${activeEnableJob.country}` : ""}
        subtitle={
          activeEnableJob
            ? `Setting up regional infrastructure for ${activeEnableJob.region}`
            : undefined
        }
        successMessage={
          activeEnableJob
            ? `${activeEnableJob.country} is ready for deployment`
            : undefined
        }
        steps={activeEnableJob?.steps ?? []}
        isComplete={isEnableComplete}
        error={enableError}
      />
    </div>
  );
}
