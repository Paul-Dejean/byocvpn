import { Instance, RegionGroup, SpawnJob } from "../../types";
import { ServerCard } from "./ServerCard";
import { SpawnJobCard } from "../jobs/SpawnJobCard";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";

interface ServerListProps {
  instances: Instance[];
  spawnJobs: SpawnJob[];
  selectedInstanceId: string | null;
  selectedJobId: string | null;
  groupedRegions: RegionGroup[];
  isLoading: boolean;
  isRefreshing: boolean;
  onSelectInstance: (instance: Instance) => void;
  onSelectSpawnJob: (spawnJob: SpawnJob) => void;
  onAddNewServer: () => void;
}

export function ServerList({
  instances,
  spawnJobs,
  selectedInstanceId,
  selectedJobId,
  groupedRegions,
  isLoading,
  isRefreshing,
  onSelectInstance,
  onSelectSpawnJob,
  onAddNewServer,
}: ServerListProps) {
  const isEmpty = instances.length === 0 && spawnJobs.length === 0;

  function findSpawnJobForInstance(instance: Instance) {
    return spawnJobs.find((spawnJob) => spawnJob.jobId === instance.spawnId);
  }

  return (
    <div className="w-fit min-w-80 flex-shrink-0 border-r border-gray-700/50 flex flex-col bg-gray-900">
      <div className="px-4 pt-4 pb-2 border-b border-gray-700/50 flex items-center gap-2">
        <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest">Servers</h2>
        {(isLoading || isRefreshing) && (
          <Spinner size="w-3 h-3" color="border-gray-500" thickness="border-2" />
        )}
      </div>
      <div className="flex-1 overflow-y-auto p-4">
        {isEmpty ? (
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
            {spawnJobs.map((spawnJob) => (
              <SpawnJobCard
                key={spawnJob.jobId}
                spawnJob={spawnJob}
                isSelected={selectedJobId === spawnJob.jobId}
                onSelect={onSelectSpawnJob}
              />
            ))}
            {instances.map((instance) => (
              <ServerCard
                key={instance.id}
                instance={instance}
                isSelected={selectedInstanceId === instance.id}
                groupedRegions={groupedRegions}
                spawnJob={findSpawnJobForInstance(instance)}
                onSelect={onSelectInstance}
              />
            ))}
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
