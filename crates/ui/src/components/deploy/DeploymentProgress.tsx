import { JobStepState } from "../../types";
import { DeploymentStepList } from "./DeploymentStepList";

interface DeploymentProgressProps {
  title: string;
  steps: JobStepState[];
  error: string | null;
}

export function DeploymentProgress({ title, steps, error }: DeploymentProgressProps) {
  return (
    <div className="flex flex-col items-center text-center gap-10">
      <div className="flex flex-col items-center gap-4">
        <span className="px-2.5 py-1 rounded-md bg-bg-medium text-caption text-fg-lighter">
          Just a moment
        </span>
        <h2 className="text-feature font-medium text-fg-lighter leading-snug">{title}</h2>
      </div>

      <div className="text-left">
        <DeploymentStepList steps={steps} />
      </div>

      {error && <p className="text-body-sm text-fg-danger-moderate max-w-[360px]">{error}</p>}
    </div>
  );
}
