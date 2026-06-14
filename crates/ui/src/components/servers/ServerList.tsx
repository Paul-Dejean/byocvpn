import { Instance, RegionGroup, SpawnJobState } from "../../types";
import { ServerCard } from "./ServerCard";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";

interface ServerListProps {

  instances: Instance[];

  selectedInstance: Instance | null;

  groupedRegions: RegionGroup[];

  isLoading: boolean;
  isRefreshing: boolean;

  getSpawnJobForInstance: (instanceId: string) => SpawnJobState | undefined;

  onSelectInstance: (instance: Instance) => void;

  onAddNewServer: () => void;
}

export function ServerList({
  instances,
  selectedInstance,
  groupedRegions,
  isLoading,
  isRefreshing,
  getSpawnJobForInstance,
  onSelectInstance,
  onAddNewServer,
}: ServerListProps) {
  return (
    <div className="w-fit min-w-80 flex-shrink-0 border-r border-gray-700/50 flex flex-col bg-gray-900">
      <div className="px-4 pt-4 pb-2 border-b border-gray-700/50">
        <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest">Servers</h2>
      </div>
      <div className="flex-1 overflow-y-auto p-4">
        {instances.length === 0 ? (
          isLoading || isRefreshing ? (
            <div className="flex justify-center py-8">
              <Spinner size="w-8 h-8" color="border-blue-500" thickness="border-4" />
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <p>No servers</p>
            </div>
          )
        ) : (
          <div className="flex flex-col gap-2">
            {(isLoading || isRefreshing) && (
              <p className="text-xs text-gray-500 text-center py-1">Refreshing server list…</p>
            )}
            {instances.map((instance) => {
              const spawnJob = getSpawnJobForInstance(instance.id);
              return (
                <ServerCard
                  key={spawnJob?.jobId ?? instance.id}
                  instance={instance}
                  isSelected={selectedInstance?.id === instance.id}
                  groupedRegions={groupedRegions}
                  spawnJob={spawnJob}
                  onSelect={onSelectInstance}
                />
              );
            })}
          </div>
        )}
      </div>

      <div className="p-4 border-t border-gray-700/50">
        <Button
          variant="primary"
          size="none"
          onClick={onAddNewServer}
          className="w-full px-4 py-3 !rounded-xl"
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
        </Button>
      </div>
    </div>
  );
}
