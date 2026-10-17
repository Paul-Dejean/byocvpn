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
    <div className="flex flex-col gap-3 rounded-xl border border-bd-moderate bg-bg-medium">
      <div className={`flex flex-col gap-4 px-4 pt-4 ${hasFailed ? "" : "pb-4"}`}>
        <div className="flex h-5 items-center justify-between gap-3">
          <ServerLocation provider={provider} region={region} />
          {hasFailed && <span className="text-caption text-fg-danger-moderate">Failed</span>}
        </div>

        <DeploymentStepList steps={steps} compact />

        {hasFailed && error && <p className="text-caption text-fg-danger-moderate">{error}</p>}
      </div>

      {hasFailed && (
        <>
          <div className="border-t border-bd-moderate" />
          <div className="px-4 pb-3">
            <Button variant="secondary" size="lg" onClick={onDismiss}>
              Dismiss
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
