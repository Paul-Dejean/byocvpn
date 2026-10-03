import { Check, Circle, Sparkles, X } from "lucide-react";
import { JobStepState, JobStepStatus } from "../../types";

interface DeploymentStepListProps {
  steps: JobStepState[];
  compact?: boolean;
}

export function DeploymentStepList({ steps, compact = false }: DeploymentStepListProps) {
  const textSize = compact ? "text-xs" : "text-sm";
  const gap = compact ? "gap-2" : "gap-4";

  return (
    <ol className={`flex flex-col ${gap}`}>
      {steps.map((step) => (
        <li key={step.id} className={`flex items-center gap-3 ${textSize}`}>
          <StepIcon status={step.status} size={compact ? 14 : 16} />
          <span className={STEP_LABEL_CLASSES[step.status]}>{step.label}</span>
        </li>
      ))}
    </ol>
  );
}

const STEP_LABEL_CLASSES: Record<JobStepStatus, string> = {
  [JobStepStatus.Pending]: "text-gray-300",
  [JobStepStatus.Running]: "text-primary",
  [JobStepStatus.Completed]: "text-gray-200",
  [JobStepStatus.Failed]: "text-danger-300",
};

function StepIcon({ status, size }: { status: JobStepStatus; size: number }) {
  switch (status) {
    case JobStepStatus.Running:
      return <Sparkles size={size} className="text-blue-400 animate-pulse flex-shrink-0" />;
    case JobStepStatus.Completed:
      return <Check size={size} className="text-success-400 flex-shrink-0" />;
    case JobStepStatus.Failed:
      return <X size={size} className="text-danger-400 flex-shrink-0" />;
    case JobStepStatus.Pending:
      return <Circle size={size} className="text-blue-400 flex-shrink-0" />;
  }
}
