import { JobStepStatus, SpawnJob, SpawnJobStatus } from "../../types";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { ServerLocation } from "../servers/ServerLocation";

interface SpawnJobCardProps {
  spawnJob: SpawnJob;
  onDismiss: (jobId: string) => void;
}

export function SpawnJobCard({ spawnJob, onDismiss }: SpawnJobCardProps) {
  const hasFailed = spawnJob.status === SpawnJobStatus.Failed;
  const runningStep = spawnJob.steps.find(
    (step) => step.status === JobStepStatus.Running,
  );
  const failedStep = spawnJob.steps.find(
    (step) => step.status === JobStepStatus.Failed,
  );
  const stepLabel = failedStep?.label ?? runningStep?.label ?? "Starting";

  return (
    <div className="rounded-xl bg-gray-750 border border-gray-500/50">
      <div className="p-3 flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3">
          <ServerLocation provider={spawnJob.provider} region={spawnJob.region} />
          {hasFailed ? (
            <span className="text-xs text-danger-400">Failed</span>
          ) : (
            <span className="flex items-center gap-1.5 text-xs text-blue-300">
              <Spinner size="w-3 h-3" color="border-blue-300" />
              Deploying
            </span>
          )}
        </div>
        <p className={`text-xs ${hasFailed ? "text-danger-300" : "text-gray-300"}`}>
          {hasFailed && spawnJob.error ? spawnJob.error : stepLabel}
        </p>
      </div>

      {hasFailed && (
        <div className="border-t border-gray-500/50 p-3">
          <Button
            variant="secondary"
            size="none"
            onClick={() => onDismiss(spawnJob.jobId)}
            className="px-3 py-1.5 text-sm"
          >
            Dismiss
          </Button>
        </div>
      )}
    </div>
  );
}
