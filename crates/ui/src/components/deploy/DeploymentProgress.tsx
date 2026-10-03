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
        <span className="px-2.5 py-1 rounded-md bg-gray-700 text-xs text-primary">
          Just a moment
        </span>
        <h2 className="text-lg font-medium text-primary leading-snug">{title}</h2>
      </div>

      <div className="text-left">
        <DeploymentStepList steps={steps} />
      </div>

      {error && <p className="text-sm text-danger-300 max-w-[360px]">{error}</p>}
    </div>
  );
}
