import { Check, Circle, Sparkles, X } from "lucide-react";
import { JobStepState, JobStepStatus } from "../../types";

interface DeploymentStepListProps {
  steps: JobStepState[];
  compact?: boolean;
}

export function DeploymentStepList({ steps, compact = false }: DeploymentStepListProps) {
  const textSize = compact ? "text-caption" : "text-body-sm";
  const gap = compact ? "gap-2" : "gap-3";

  return (
    <ol className={`flex flex-col ${gap}`}>
      {steps.map((step) => (
        <li key={step.id} className={`flex items-center gap-2 ${textSize}`}>
          <StepIcon status={step.status} size={14} />
          <span className={STEP_LABEL_CLASSES[step.status]}>{step.label}</span>
        </li>
      ))}
    </ol>
  );
}

const STEP_LABEL_CLASSES: Record<JobStepStatus, string> = {
  [JobStepStatus.Pending]: "text-fg-medium",
  [JobStepStatus.Running]: "text-fg-lighter",
  [JobStepStatus.Completed]: "text-fg-medium",
  [JobStepStatus.Failed]: "text-fg-danger-moderate",
};

function StepIcon({ status, size }: { status: JobStepStatus; size: number }) {
  switch (status) {
    case JobStepStatus.Running:
      return <Sparkles size={size} className="shrink-0 animate-pulse text-bd-brand" />;
    case JobStepStatus.Completed:
      return <Check size={size} className="shrink-0 text-fg-success-moderate" />;
    case JobStepStatus.Failed:
      return <X size={size} className="shrink-0 text-fg-danger-moderate" />;
    case JobStepStatus.Pending:
      return <Circle size={size} className="shrink-0 text-bd-brand" />;
    default:
      return null;
  }
}
