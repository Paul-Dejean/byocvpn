import { Instance, RegionGroup, SpawnJobState } from "../../types";
import { ServerCard } from "./ServerCard";

interface ServerListProps {

  instances: Instance[];

  selectedInstance: Instance | null;

  groupedRegions: RegionGroup[];

  isLoading: boolean;

  getSpawnJobForInstance: (instanceId: string) => SpawnJobState | undefined;

  onSelectInstance: (instance: Instance) => void;

  onAddNewServer: () => void;
}

export function ServerList({
  instances,
  selectedInstance,
  groupedRegions,
  isLoading,
  getSpawnJobForInstance,
  onSelectInstance,
  onAddNewServer,
}: ServerListProps) {
  return (
    <div className="w-80 flex-shrink-0 border-r border-gray-700/50 flex flex-col">
      <div className="px-4 pt-4 pb-2 border-b border-gray-700/50">
        <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest">Servers</h2>
      </div>
      <div className="flex-1 overflow-y-auto p-4">
        {isLoading && (
          <div className="flex justify-center py-8">
            <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        )}

        {!isLoading && instances.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <p className="mb-4">No servers running</p>
            <p className="text-sm">Click "Add New Server" to get started</p>
          </div>
        ) : (
          <div className="space-y-2">
            {}
            {instances.map((instance) => (
              <ServerCard
                key={instance.id}
                instance={instance}
                isSelected={selectedInstance?.id === instance.id}
                groupedRegions={groupedRegions}
                spawnJob={getSpawnJobForInstance(instance.id)}
                onSelect={onSelectInstance}
              />
            ))}
          </div>
        )}
      </div>

      {}
      <div className="p-4 border-t border-gray-700/50">
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
    </div>
  );
}
