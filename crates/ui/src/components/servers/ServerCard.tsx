import { Instance, RegionGroup, SpawnJobState } from "../../types";
import { getRegionInfo } from "../../types/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";

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

const PROVIDER_STRIPE: Record<string, string> = {
  aws: "border-l-orange-500",
  oracle: "border-l-red-500",
  gcp: "border-l-blue-500",
  azure: "border-l-sky-500",
};

export function ServerCard({
  instance,
  isSelected,
  groupedRegions: _groupedRegions,
  spawnJob,
  onSelect,
}: ServerCardProps) {
  const regionInfo = getRegionInfo(instance.provider, instance.region ?? "");
  const stripeColor = PROVIDER_STRIPE[instance.provider] ?? "border-l-gray-600";

  const isSpawning = instance.state === "spawning";

  const runningStep = spawnJob?.steps.find((s) => s.status === "running");
  const stepLabel = runningStep?.label ?? (isSpawning ? "Starting…" : null);

  return (
    <button
      onClick={() => onSelect(instance)}
      className={`text-left p-3 rounded-lg transition-all border border-l-4 ${stripeColor} ${
        isSpawning
          ? isSelected
            ? "bg-blue-700/60 text-white glow-accent border-blue-500/40"
            : "bg-gray-800 text-gray-300 hover:bg-gray-750 border-white/10"
          : isSelected
            ? "bg-blue-600/80 text-white glow-accent border-blue-500/40"
            : "bg-gray-800 hover:bg-gray-700 text-gray-200 border-white/10"
      }`}
    >
      <div className={`flex items-center justify-between gap-4 ${isSpawning ? "mb-2" : ""}`}>
        <div className="flex items-center gap-3">
          <FlagIcon countryCode={regionInfo.countryCode} className="text-xl flex-shrink-0" />
          <div>
            <p className="font-medium text-sm">
              {regionInfo.city || instance.name || "VPN Server"}
            </p>
            <p className="text-xs opacity-75 font-mono">{instance.region}</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <ProviderIcon provider={instance.provider} className="w-6 h-6" />
          <span
            className={`px-2 py-1 rounded text-xs font-medium ${
              isSpawning
                ? "bg-blue-900/50 text-blue-300 flex items-center gap-1"
                : instance.state === "running"
                  ? "bg-green-900/50 text-green-300"
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
      {isSpawning && (
        <p className="text-xs font-mono opacity-75 truncate">
          {stepLabel ?? "Starting…"}
        </p>
      )}
    </button>
  );
}
