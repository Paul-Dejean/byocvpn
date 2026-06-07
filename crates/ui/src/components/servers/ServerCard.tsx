import { Instance, InstanceState, RegionGroup, SpawnJobState, JobStepStatus } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { CloudProviderName } from "../../types";
import { Spinner } from "../common/Spinner";

interface ServerCardProps {
  instance: Instance;
  isSelected: boolean;
  groupedRegions: RegionGroup[];
  spawnJob?: SpawnJobState;
  onSelect: (instance: Instance) => void;
}

const PROVIDER_STRIPE: Record<CloudProviderName, string> = {
  [CloudProviderName.Aws]: "border-l-orange-500",
  [CloudProviderName.Oracle]: "border-l-red-500",
  [CloudProviderName.Gcp]: "border-l-blue-500",
  [CloudProviderName.Azure]: "border-l-sky-500",
};

const STATE_BADGE: Record<InstanceState, { className: string; label: string; spinner?: boolean }> = {
  [InstanceState.Spawning]:   { className: "bg-blue-900/50 text-blue-300",    label: "spawning",   spinner: true },
  [InstanceState.Installing]: { className: "bg-yellow-900/50 text-yellow-300", label: "installing", spinner: true },
  [InstanceState.Error]:      { className: "bg-red-900/50 text-red-400",      label: "error" },
  [InstanceState.Running]:    { className: "bg-green-900/50 text-green-300",  label: "running" },
  [InstanceState.Creating]:   { className: "bg-yellow-900/50 text-yellow-300", label: "creating" },
  [InstanceState.Stopping]:   { className: "bg-red-900/50 text-red-300",      label: "stopping" },
  [InstanceState.Deleting]:   { className: "bg-red-900/50 text-red-300",      label: "deleting" },
  [InstanceState.Stopped]:    { className: "bg-gray-700/50 text-gray-500",    label: "stopped" },
  [InstanceState.Deleted]:    { className: "bg-gray-700/50 text-gray-500",    label: "deleted" },
  [InstanceState.Unknown]:    { className: "bg-gray-900/50 text-gray-400",    label: "unknown" },
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

  const isInProgress =
    instance.state === InstanceState.Spawning || instance.state === InstanceState.Installing;

  const badge = STATE_BADGE[instance.state] ?? {
    className: "bg-gray-900/50 text-gray-400",
    label: instance.state,
  };

  const runningStep = spawnJob?.steps.find((step) => step.status === JobStepStatus.Running);
  const stepLabel = runningStep?.label ?? (isInProgress ? "Starting…" : null);

  return (
    <button
      onClick={() => onSelect(instance)}
      className={`text-left p-3 rounded-lg transition-all border border-l-4 ${stripeColor} ${
        isInProgress
          ? isSelected
            ? "bg-blue-700/60 text-white glow-accent border-blue-500/40"
            : "bg-gray-800 text-gray-300 hover:bg-gray-750 border-white/10"
          : isSelected
            ? "bg-blue-600/80 text-white glow-accent border-blue-500/40"
            : "bg-gray-800 hover:bg-gray-700 text-gray-200 border-white/10"
      }`}
    >
      <div className={`flex items-center justify-between gap-4 ${isInProgress ? "mb-2" : ""}`}>
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
          <span className={`px-2 py-1 rounded text-xs font-medium flex items-center gap-1 ${badge.className}`}>
            {badge.spinner && <Spinner />}
            {badge.label}
          </span>
        </div>
      </div>
      {isInProgress && stepLabel && (
        <p className="text-xs font-mono opacity-75 truncate">{stepLabel}</p>
      )}
    </button>
  );
}
