import { JobStepState } from "../../types";
import { DeploymentStepList } from "./DeploymentStepList";

interface DeploymentProgressProps {
  title: string;
  steps: JobStepState[];
  error: string | null;
}

export function DeploymentProgress({ title, steps, error }: DeploymentProgressProps) {
  return (
    <div className="flex flex-col items-center gap-8 text-center">
      <div className="flex flex-col items-center gap-3">
        <span className="rounded-md bg-bg-medium px-2 py-0.5 text-caption text-fg-lighter">
          Just a moment
        </span>
        <h2 className="text-feature font-semibold text-fg-lighter">{title}</h2>
      </div>

      <div className="text-left">
        <DeploymentStepList steps={steps} />
      </div>

      {error && <p className="max-w-90 text-body-sm text-fg-danger-moderate">{error}</p>}
    </div>
  );
}
