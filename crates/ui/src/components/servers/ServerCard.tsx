import {
  Instance,
  InstanceState,
  RegionGroup,
  SpawnJobState,
  JobStepStatus,
} from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { CloudProviderName } from "../../types";
import { Badge, BadgeVariant } from "../primitives/Badge";
import { SelectableCard } from "../primitives/SelectableCard";

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

const STATE_BADGE: Record<
  InstanceState,
  { variant: BadgeVariant; label: string; spinner?: boolean }
> = {
  [InstanceState.Spawning]: {
    variant: "info",
    label: "spawning",
    spinner: true,
  },
  [InstanceState.Installing]: {
    variant: "warning",
    label: "installing",
    spinner: true,
  },
  [InstanceState.Error]: {
    variant: "danger",
    label: "error",
  },
  [InstanceState.Running]: {
    variant: "success",
    label: "running",
  },
  [InstanceState.Stopping]: {
    variant: "danger",
    label: "stopping",
  },
  [InstanceState.Stopped]: {
    variant: "neutral",
    label: "stopped",
  },
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
    instance.state === InstanceState.Spawning ||
    instance.state === InstanceState.Installing;

  const isInteractive = [
    InstanceState.Spawning,
    InstanceState.Installing,
    InstanceState.Running,
    InstanceState.Error,
  ].includes(instance.state);

  const badge = STATE_BADGE[instance.state] ?? {
    variant: "neutral",
    label: instance.state,
  };

  const runningStep = spawnJob?.steps.find(
    (step) => step.status === JobStepStatus.Running,
  );
  const stepLabel = runningStep?.label ?? (isInProgress ? "Starting…" : null);

  return (
    <SelectableCard
      onClick={() => isInteractive && onSelect(instance)}
      disabled={!isInteractive}
      className={`p-3 rounded-lg border border-l-4 ${stripeColor} ${
        !isInteractive
          ? "bg-gray-800/50 text-gray-400 cursor-not-allowed border-gray-500/15 opacity-80"
          : isInProgress
            ? isSelected
              ? "bg-blue-700/60 text-white glow-accent border-blue-500/40"
              : "bg-gray-800 text-gray-300 hover:bg-gray-750 border-gray-500/25"
            : isSelected
              ? "bg-blue-600/80 text-white glow-accent border-blue-500/40"
              : "bg-gray-800 hover:bg-gray-700 text-gray-200 border-gray-500/25"
      }`}
    >
      <div
        className={`flex items-center justify-between gap-4 ${isInProgress ? "mb-2" : ""}`}
      >
        <div className="flex items-center gap-3">
          <FlagIcon
            countryCode={regionInfo.countryCode}
            className="text-xl flex-shrink-0"
          />
          <div>
            <p className="font-medium text-sm">
              {regionInfo.city || instance.name || "VPN Server"}
            </p>
            <p className="text-xs opacity-75 font-mono">{instance.region}</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <ProviderIcon provider={instance.provider} className="w-6 h-6" />
          <Badge variant={badge.variant} spinner={badge.spinner}>
            {badge.label}
          </Badge>
        </div>
      </div>
      {isInProgress && stepLabel && (
        <p className="text-xs font-mono opacity-75 truncate">{stepLabel}</p>
      )}
    </SelectableCard>
  );
}
