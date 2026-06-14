import { CloudProviderName, JobStepState, JobStepStatus } from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { Drawer } from "../primitives/Drawer";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";
import { Alert } from "../primitives/Alert";

interface JobProgressDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  provider: CloudProviderName;
  title?: string;
  subtitle?: string;
  steps: JobStepState[];
  isComplete: boolean;
  successMessage?: string;
  error: string | null;
}

function StepStatusIcon({ status }: { status: JobStepStatus }) {
  switch (status) {
    case JobStepStatus.Running:
      return <Spinner size="w-5 h-5" color="border-blue-400" />;
    case JobStepStatus.Completed:
      return (
        <svg
          className="w-5 h-5 text-success-400 flex-shrink-0"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        </svg>
      );
    case JobStepStatus.Failed:
      return (
        <svg
          className="w-5 h-5 text-danger-400 flex-shrink-0"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      );
    default:
      return <div className="w-5 h-5 rounded-full border-2 border-gray-600 flex-shrink-0" />;
  }
}

export function JobProgressDrawer({
  isOpen,
  onClose,
  provider,
  title,
  subtitle,
  steps,
  isComplete,
  successMessage,
  error,
}: JobProgressDrawerProps) {
  return (
    <Drawer
      isOpen={isOpen}
      onClose={onClose}
      title={title ?? `Provisioning ${PROVIDER_METADATA[provider].shortLabel}`}
      subtitle={subtitle ?? "Setting up your account infrastructure"}
      footer={
        <Button variant="secondary" onClick={onClose} className="w-full">
          Close
        </Button>
      }
    >
      <div className="space-y-4">
        {steps.map((step, index) => (
          <div key={step.id} className="flex items-start gap-3">
            <div className="mt-0.5">
              <StepStatusIcon status={step.status} />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500 font-mono tabular-nums">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span
                  className={`text-sm font-medium ${
                    step.status === JobStepStatus.Completed
                      ? "text-gray-300"
                      : step.status === JobStepStatus.Running
                        ? "text-primary"
                        : step.status === JobStepStatus.Failed
                          ? "text-danger-300"
                          : "text-gray-500"
                  }`}
                >
                  {step.label}
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>

      {isComplete && (
        <Alert
          variant="success"
          className="mt-6"
          icon={
            <svg
              className="w-5 h-5 text-success-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          }
          title={successMessage ?? "Account provisioned successfully"}
        />
      )}

      {error && (
        <Alert
          variant="error"
          className="mt-6"
          icon={
            <svg
              className="w-5 h-5 text-danger-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          }
          title="Provisioning failed"
        >
          {error}
        </Alert>
      )}
    </Drawer>
  );
}
