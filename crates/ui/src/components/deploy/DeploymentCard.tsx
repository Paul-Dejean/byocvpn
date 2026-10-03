import { CloudProviderName, JobStepState } from "../../types";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { ServerLocation } from "../servers/ServerLocation";
import { DeploymentStepList } from "./DeploymentStepList";

interface DeploymentCardProps {
  provider: CloudProviderName;
  region: string;
  steps: JobStepState[];
  hasFailed: boolean;
  error: string | null;
  onDismiss: () => void;
}

export function DeploymentCard({
  provider,
  region,
  steps,
  hasFailed,
  error,
  onDismiss,
}: DeploymentCardProps) {
  return (
    <div className="rounded-xl bg-gray-750 border border-gray-500/50">
      <div className="p-3 flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3">
          <ServerLocation provider={provider} region={region} />
          {hasFailed ? (
            <span className="text-xs text-danger-400">Failed</span>
          ) : (
            <span className="flex items-center gap-1.5 text-xs text-blue-300">
              <Spinner size="w-3 h-3" color="border-blue-300" />
              Deploying
            </span>
          )}
        </div>

        {steps.length > 0 ? (
          <DeploymentStepList steps={steps} compact />
        ) : (
          <p className="text-xs text-gray-300">Starting</p>
        )}

        {hasFailed && error && (
          <p className="text-xs text-danger-300">{error}</p>
        )}
      </div>

      {hasFailed && (
        <div className="border-t border-gray-500/50 p-3">
          <Button
            variant="secondary"
            size="none"
            onClick={onDismiss}
            className="px-3 py-1.5 text-sm"
          >
            Dismiss
          </Button>
        </div>
      )}
    </div>
  );
}
