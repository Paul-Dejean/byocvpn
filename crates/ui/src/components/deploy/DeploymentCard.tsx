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
    <div className="rounded-xl bg-bg-medium border border-bd-moderate flex flex-col gap-3">
      <div className={`px-4 pt-4 flex flex-col gap-4 ${hasFailed ? "" : "pb-4"}`}>
        <div className="flex items-center justify-between gap-3 h-5">
          <ServerLocation provider={provider} region={region} />
          {hasFailed && <span className="text-caption text-fg-danger-moderate">Failed</span>}
        </div>

        <DeploymentStepList steps={steps} compact />

        {hasFailed && error && (
          <p className="text-caption text-fg-danger-moderate">{error}</p>
        )}
      </div>

      {hasFailed && (
        <>
          <div className="border-t border-bd-moderate" />
          <div className="px-4 pb-3">
            <Button
              variant="secondary"
              size="lg"
              onClick={onDismiss}
            >
              Dismiss
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
