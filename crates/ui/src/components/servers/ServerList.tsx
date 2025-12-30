import { Instance, RegionGroup } from "../../types";
import { ServerCard } from "./ServerCard";

/**
 * Props for the ServerList component
 */
interface ServerListProps {
  /** List of server instances to display */
  instances: Instance[];
  /** Currently selected instance */
  selectedInstance: Instance | null;
  /** Grouped regions for flag lookup */
  groupedRegions: RegionGroup[];
  /** Whether the list is loading */
  isLoading: boolean;
  /** Callback when an instance is selected */
  onSelectInstance: (instance: Instance) => void;
  /** Callback when add new server button is clicked */
  onAddNewServer: () => void;
}

/**
 * Displays a list of server instances with an add button
 */
export function ServerList({
  instances,
  selectedInstance,
  groupedRegions,
  isLoading,
  onSelectInstance,
  onAddNewServer,
}: ServerListProps) {
  return (
    <div className="w-96 bg-gray-800 border-r border-gray-700 flex flex-col">
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
    </div>
  );
}
