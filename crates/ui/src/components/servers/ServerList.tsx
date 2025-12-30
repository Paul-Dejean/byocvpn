import { ExistingInstance, RegionGroup } from "../../types";
import { LoadingSpinner } from "../common/LoadingSpinner";
import { ServerCard } from "./ServerCard";

interface ServerListProps {
  instances: ExistingInstance[];
  selectedInstance: ExistingInstance | null;
  groupedRegions: RegionGroup[];
  isLoading: boolean;
  onSelectInstance: (instance: ExistingInstance) => void;
  onAddNewServer: () => void;
  spawningRegions: string[];
}

export function ServerList({
  instances,
  selectedInstance,
  groupedRegions,
  isLoading,
  onSelectInstance,
  onAddNewServer,
  spawningRegions,
}: ServerListProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((group) => group.regions)
      .find((region) => region.name === regionName) as any;
    return region?.flag || "🌍";
  };

  return (
    <div className="w-96 bg-gray-800 border-r border-gray-700 flex flex-col">
      {isLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <LoadingSpinner message="Loading servers..." />
        </div>
      ) : (
        <>
          <div className="flex-1 overflow-y-auto p-4">
            {isLoading && (
              <div className="flex justify-center py-8">
                <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
              </div>
            )}

            {!isLoading &&
            instances.length === 0 &&
            spawningRegions.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <p className="mb-4">No servers running</p>
                <p className="text-sm">Click "Add New Server" to get started</p>
              </div>
            ) : (
              <div className="space-y-2">
                {/* Spawning placeholder cards - one for each region */}
                {spawningRegions.map((regionName) => (
                  <div
                    key={`spawning-${regionName}`}
                    className="w-full p-3 rounded-lg bg-gray-700 border-2 border-dashed border-blue-500 opacity-75"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <span className="text-lg">
                          {getRegionFlag(regionName)}
                        </span>
                        <div>
                          <p className="font-medium text-sm text-gray-300">
                            Deploying...
                          </p>
                          <p className="text-xs text-gray-400">{regionName}</p>
                        </div>
                      </div>
                      <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                    </div>
                    <p className="text-xs font-mono text-gray-400">
                      Setting up server...
                    </p>
                  </div>
                ))}

                {/* Existing server instances */}
                {instances.map((instance) => (
                  <ServerCard
                    key={instance.id}
                    instance={instance}
                    isSelected={selectedInstance?.id === instance.id}
                    groupedRegions={groupedRegions}
                    onSelect={onSelectInstance}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Add Server Button - Fixed at bottom */}
          <div className="p-4 border-t border-gray-700">
            <button
              onClick={onAddNewServer}
              className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition font-medium flex items-center justify-center gap-2 shadow-lg hover:shadow-xl"
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
                  d="M12 4v16m8-8H4"
                />
              </svg>
              Add New Server
            </button>
          </div>
        </>
      )}
    </div>
  );
}
