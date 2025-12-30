import { Instance, RegionGroup } from "../../types";

interface ServerCardProps {
  instance: Instance;
  isSelected: boolean;
  groupedRegions: RegionGroup[];
  onSelect: (instance: Instance) => void;
}

function MiniSpinner() {
  return (
    <div className="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
  );
}

export function ServerCard({
  instance,
  isSelected,
  groupedRegions,
  onSelect,
}: ServerCardProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === regionName) as any;
    return region?.flag || "🌍";
  };

  const isSpawning = instance.state === "spawning";

  return (
    <button
      onClick={() => !isSpawning && onSelect(instance)}
      disabled={isSpawning}
      className={`w-full text-left p-3 rounded-lg transition-all ${
        isSpawning
          ? "bg-gray-800 text-gray-400 cursor-wait opacity-75"
          : isSelected
            ? "bg-blue-600 text-white shadow-lg"
            : "bg-gray-700 hover:bg-gray-600 text-gray-200"
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <span className="text-lg">{getRegionFlag(instance.region)}</span>
          <div>
            <p className="font-medium text-sm">
              {instance.name || "VPN Server"}
            </p>
            <p className="text-xs opacity-75">{instance.region}</p>
          </div>
        </div>
        <span
          className={`px-2 py-1 rounded text-xs font-medium ${
            isSpawning
              ? "bg-blue-900/50 text-blue-300 flex items-center gap-1"
              : instance.state === "running"
                ? "bg-green-900/50 text-green-300"
                : "bg-gray-900/50 text-gray-400"
          }`}
        >
          {isSpawning && <MiniSpinner />}
          {instance.state}
        </span>
      </div>
      <p className="text-xs font-mono opacity-75 truncate">
        {isSpawning ? "Creating server instance..." : instance.publicIpV4}
      </p>
    </button>
  );
}
