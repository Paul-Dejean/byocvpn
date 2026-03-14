import { Instance, RegionGroup, SpawnJobState } from "../../types";

interface ProviderBadgeProps {
  provider: string;
}

function ProviderBadge({ provider }: ProviderBadgeProps) {
  if (provider === "aws") {
    return (
      <span className="px-1.5 py-0.5 rounded text-xs font-bold bg-orange-900/50 text-orange-300">
        AWS
      </span>
    );
  }
  if (provider === "oracle") {
    return (
      <span className="px-1.5 py-0.5 rounded text-xs font-bold bg-red-900/50 text-red-300">
        OCI
      </span>
    );
  }
  if (provider === "gcp") {
    return (
      <span className="px-1.5 py-0.5 rounded text-xs font-bold bg-blue-900/50 text-blue-300">
        GCP
      </span>
    );
  }
  if (provider === "azure") {
    return (
      <span className="px-1.5 py-0.5 rounded text-xs font-bold bg-sky-900/50 text-sky-300">
        AZ
      </span>
    );
  }
  return (
    <span className="px-1.5 py-0.5 rounded text-xs font-bold bg-gray-700 text-gray-400">
      ?
    </span>
  );
}

interface ServerCardProps {
  instance: Instance;
  isSelected: boolean;
  groupedRegions: RegionGroup[];

  spawnJob?: SpawnJobState;
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
  spawnJob,
  onSelect,
}: ServerCardProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === regionName) as any;
    return region?.flag || "🌍";
  };

  const isSpawning = instance.state === "spawning";

  const runningStep = spawnJob?.steps.find((s) => s.status === "running");
  const stepLabel = runningStep?.label ?? (isSpawning ? "Starting…" : null);

  return (
    <button
      onClick={() => onSelect(instance)}
      className={`w-full text-left p-3 rounded-lg transition-all ${
        isSpawning
          ? isSelected
            ? "bg-blue-700/60 text-white shadow-lg"
            : "bg-gray-800 text-gray-300 hover:bg-gray-750"
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
        <div className="flex items-center gap-1.5">
          <ProviderBadge provider={instance.provider} />
          <span
            className={`px-2 py-1 rounded text-xs font-medium ${
              isSpawning
                ? "bg-blue-900/50 text-blue-300 flex items-center gap-1"
                : instance.state === "running"
                  ? "bg-orange-900/50 text-orange-300"
                  : instance.state === "creating"
                    ? "bg-yellow-900/50 text-yellow-300"
                    : instance.state === "stopping" ||
                        instance.state === "deleting"
                      ? "bg-red-900/50 text-red-300"
                      : instance.state === "stopped" ||
                          instance.state === "deleted"
                        ? "bg-gray-700/50 text-gray-500"
                        : "bg-gray-900/50 text-gray-400"
            }`}
          >
            {isSpawning && <MiniSpinner />}
            {isSpawning ? "deploying" : instance.state}
          </span>
        </div>
      </div>
      <p className="text-xs font-mono opacity-75 truncate">
        {isSpawning ? (stepLabel ?? "Starting…") : instance.publicIpV4}
      </p>
    </button>
  );
}
