import { JobStepStatus, SpawnJob, SpawnJobStatus } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Badge } from "../primitives/Badge";
import { SelectableCard } from "../primitives/SelectableCard";
import { PROVIDER_STRIPE } from "../servers/providerStripe";

interface SpawnJobCardProps {
  spawnJob: SpawnJob;
  isSelected: boolean;
  onSelect: (spawnJob: SpawnJob) => void;
}

export function SpawnJobCard({
  spawnJob,
  isSelected,
  onSelect,
}: SpawnJobCardProps) {
  const regionInfo = getRegionInfo(spawnJob.provider, spawnJob.region);
  const stripeColor = PROVIDER_STRIPE[spawnJob.provider] ?? "border-l-gray-600";
  const hasFailed = spawnJob.status === SpawnJobStatus.Failed;

  const runningStep = spawnJob.steps.find(
    (step) => step.status === JobStepStatus.Running,
  );
  const failedStep = spawnJob.steps.find(
    (step) => step.status === JobStepStatus.Failed,
  );
  const stepLabel = failedStep?.label ?? runningStep?.label ?? "Starting…";

  return (
    <SelectableCard
      onClick={() => onSelect(spawnJob)}
      className={`p-3 rounded-lg border border-l-4 ${stripeColor} mb-2 ${
        isSelected
          ? "bg-blue-700/60 text-white glow-accent border-blue-500/40"
          : "bg-gray-800 text-gray-300 hover:bg-gray-750 border-gray-500/25"
      }`}
    >
      <div className="flex items-center justify-between gap-4 mb-2">
        <div className="flex items-center gap-3">
          <FlagIcon
            countryCode={regionInfo.countryCode}
            className="text-xl flex-shrink-0"
          />
          <div>
            <p className="font-medium text-sm">
              {regionInfo.city || "VPN Server"}
            </p>
            <p className="text-xs opacity-75 font-mono">{spawnJob.region}</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <ProviderIcon provider={spawnJob.provider} className="w-6 h-6" />
          <Badge variant={hasFailed ? "danger" : "info"} spinner={!hasFailed}>
            {hasFailed ? "failed" : "deploying"}
          </Badge>
        </div>
      </div>
      <p className="text-xs font-mono opacity-75 truncate">{stepLabel}</p>
    </SelectableCard>
  );
}
