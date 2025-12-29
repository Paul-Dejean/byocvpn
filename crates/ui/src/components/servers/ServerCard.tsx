import { ExistingInstance, RegionGroup } from "../../types";

interface ServerCardProps {
  instance: ExistingInstance;
  isSelected: boolean;
  groupedRegions: RegionGroup[];
  onSelect: (instance: ExistingInstance) => void;
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

  return (
    <button
      onClick={() => onSelect(instance)}
      className={`w-full text-left p-3 rounded-lg transition-all ${
        isSelected
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
            instance.state === "running"
              ? "bg-green-900/50 text-green-300"
              : "bg-gray-900/50 text-gray-400"
          }`}
        >
          {instance.state}
        </span>
      </div>
      <p className="text-xs font-mono opacity-75 truncate">
        {instance.public_ip_v4}
      </p>
    </button>
  );
}
