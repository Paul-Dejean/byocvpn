import { JobStepState, JobStepStatus } from "../../types";
import { Spinner } from "../primitives/Spinner";

interface JobStepListProps {
  steps: JobStepState[];
}

export function JobStepList({ steps }: JobStepListProps) {
  return (
    <div className="p-3 space-y-1">
      {steps.map((step) => (
        <div
          key={step.id}
          className={`flex items-center gap-3 p-2.5 rounded-lg ${STEP_ROW_STYLE[step.status]}`}
        >
          <StepIndicator status={step.status} />
          <div className="flex-1 min-w-0">
            <p className={`text-sm ${STEP_LABEL_STYLE[step.status]}`}>
              {step.label}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}

const STEP_ROW_STYLE: Record<JobStepStatus, string> = {
  [JobStepStatus.Pending]: "opacity-40",
  [JobStepStatus.Running]: "bg-blue-900/30 border border-blue-700/40",
  [JobStepStatus.Completed]: "opacity-60",
  [JobStepStatus.Failed]: "bg-danger-900/20 border border-danger-700/40",
};

const STEP_LABEL_STYLE: Record<JobStepStatus, string> = {
  [JobStepStatus.Pending]: "text-gray-500",
  [JobStepStatus.Running]: "text-blue-300 font-medium",
  [JobStepStatus.Completed]: "text-gray-400",
  [JobStepStatus.Failed]: "text-danger-300",
};

function StepIndicator({ status }: { status: JobStepStatus }) {
  if (status === JobStepStatus.Running) {
    return <Spinner size="w-5 h-5" color="border-blue-400" />;
  }

  if (status === JobStepStatus.Completed) {
    return (
      <svg
        className="w-5 h-5 text-success-400 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fillRule="evenodd"
          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
          clipRule="evenodd"
        />
      </svg>
    );
  }

  if (status === JobStepStatus.Failed) {
    return (
      <svg
        className="w-5 h-5 text-danger-400 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fillRule="evenodd"
          d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
          clipRule="evenodd"
        />
      </svg>
    );
  }

  return (
    <div className="w-5 h-5 rounded-full border-2 border-gray-600 flex-shrink-0" />
  );
}
