import { Check, Circle, Server, Sparkles, X } from "lucide-react";
import { JobStepState, JobStepStatus } from "../../types";

interface DeploymentProgressProps {
  title: string;
  steps: JobStepState[];
  error: string | null;
  showIllustration?: boolean;
  centered?: boolean;
}

export function DeploymentProgress({
  title,
  steps,
  error,
  showIllustration = true,
  centered = false,
}: DeploymentProgressProps) {
  const alignment = centered ? "items-center text-center" : "items-start";

  return (
    <div className={`flex flex-col gap-10 ${alignment}`}>
      <div className={`flex flex-col gap-4 ${alignment}`}>
        <span className="px-2.5 py-1 rounded-md bg-gray-700 text-xs text-primary">
          Just a moment
        </span>
        <h2
          className={`text-lg font-medium text-primary leading-snug ${
            centered ? "" : "max-w-[220px]"
          }`}
        >
          {title}
        </h2>
      </div>

      <div className="flex items-start gap-16">
        <ol className="flex flex-col gap-4">
          {steps.map((step) => (
            <li key={step.id} className="flex items-center gap-3 text-sm">
              <StepIcon status={step.status} />
              <span className={STEP_LABEL_CLASSES[step.status]}>{step.label}</span>
            </li>
          ))}
        </ol>

        {showIllustration && (
          <div className="w-[194px] h-[140px] rounded-xl border border-dashed border-gray-500 flex items-center justify-center">
            <Server size={64} strokeWidth={1.25} className="text-gray-500" />
          </div>
        )}
      </div>

      {error && <p className="text-sm text-danger-300">{error}</p>}
    </div>
  );
}

const STEP_LABEL_CLASSES: Record<JobStepStatus, string> = {
  [JobStepStatus.Pending]: "text-gray-300",
  [JobStepStatus.Running]: "text-primary",
  [JobStepStatus.Completed]: "text-gray-200",
  [JobStepStatus.Failed]: "text-danger-300",
};

function StepIcon({ status }: { status: JobStepStatus }) {
  switch (status) {
    case JobStepStatus.Running:
      return <Sparkles size={16} className="text-blue-400 animate-pulse" />;
    case JobStepStatus.Completed:
      return <Check size={16} className="text-success-400" />;
    case JobStepStatus.Failed:
      return <X size={16} className="text-danger-400" />;
    case JobStepStatus.Pending:
      return <Circle size={16} className="text-blue-400" />;
  }
}
