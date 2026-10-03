import { CloudProviderName, JobStepState } from "../../types";
import { Button } from "../primitives/Button";
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
    <div className="rounded-xl bg-gray-700 border border-gray-500/60 flex flex-col gap-3">
      <div className={`px-4 pt-4 flex flex-col gap-4 ${hasFailed ? "" : "pb-4"}`}>
        <div className="flex items-center justify-between gap-3 h-5">
          <ServerLocation provider={provider} region={region} />
          {hasFailed && <span className="text-xs text-danger-400">Failed</span>}
        </div>

        <DeploymentStepList steps={steps} compact />

        {hasFailed && error && (
          <p className="text-xs text-danger-300">{error}</p>
        )}
      </div>

      {hasFailed && (
        <>
          <div className="border-t border-gray-500/60" />
          <div className="px-4 pb-3">
            <Button
              variant="secondary"
              size="none"
              onClick={onDismiss}
              className="h-7 px-3 text-sm"
            >
              Dismiss
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
