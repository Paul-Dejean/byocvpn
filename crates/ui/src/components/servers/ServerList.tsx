import { ExistingInstance, RegionGroup } from "../../types";
import { LoadingSpinner } from "../common/LoadingSpinner";
import { ServerCard } from "./ServerCard";

interface ServerListProps {
  instances: ExistingInstance[];
  selectedInstance: ExistingInstance | null;
  groupedRegions: RegionGroup[];
  isLoading: boolean;
  error: string | null;
  onSelectInstance: (instance: ExistingInstance) => void;
  onAddNewServer: () => void;
  isSpawning?: boolean;
  spawningRegion?: string;
}

export function ServerList({
  instances,
  selectedInstance,
  groupedRegions,
  isLoading,
  error,
  onSelectInstance,
  onAddNewServer,
  isSpawning,
  spawningRegion,
}: ServerListProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === regionName) as any;
    return region?.flag || "🌍";
  };

  return (
    <div className="w-96 bg-gray-800 border-r border-gray-700 flex flex-col">
      <div className="p-4 border-b border-gray-700">
        <h2 className="text-xl font-semibold text-blue-400">Your Servers</h2>
        <p className="text-sm text-gray-400 mt-1">
          {instances.length} server{instances.length !== 1 ? "s" : ""}
        </p>
      </div>

      {isLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <LoadingSpinner message="Loading servers..." />
        </div>
      ) : (
        <>
          <div className="flex-1 overflow-y-auto p-4">
            {instances.length === 0 ? (
              <div className="text-center py-12">
                <svg
                  className="w-16 h-16 text-gray-600 mx-auto mb-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1}
                    d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
                  />
                </svg>
                <p className="text-gray-400 text-sm">No servers deployed yet</p>
                <p className="text-gray-500 text-xs mt-1">
                  Click the button below to add one
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                {/* Spawning placeholder card */}
                {isSpawning && spawningRegion && (
                  <div className="w-full p-3 rounded-lg bg-gray-700 border-2 border-dashed border-blue-500 opacity-75">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <span className="text-lg">
                          {getRegionFlag(spawningRegion)}
                        </span>
                        <div>
                          <p className="font-medium text-sm text-gray-300">
                            Deploying...
                          </p>
                          <p className="text-xs text-gray-400">{spawningRegion}</p>
                        </div>
                      </div>
                      <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                    </div>
                    <p className="text-xs font-mono text-gray-400">
                      Setting up server...
                    </p>
                  </div>
                )}

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

            {error && (
              <div className="mt-4 p-3 bg-red-900/20 border border-red-500/50 rounded-lg">
                <p className="text-red-300 text-xs">{error}</p>
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
