import { Check, Circle, Sparkles, X } from "lucide-react";
import { CloudProviderName, JobStepState, JobStepStatus } from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { Drawer } from "../primitives/Drawer";
import { Button } from "../primitives/Button";
import { Banner } from "../primitives/Banner";

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
        <Button variant="secondary" size="lg" onClick={onClose} className="w-full">
          Close
        </Button>
      }
    >
      <div className="flex flex-col gap-6">
        <ProvisionSteps steps={steps} />

        {isComplete && (
          <Banner
            variant="success"
            icon={<Check size={16} className="text-fg-success-moderate" />}
            title={successMessage ?? "Account provisioned successfully"}
          />
        )}

        {error && (
          <Banner
            variant="danger"
            icon={<X size={16} className="text-fg-danger-moderate" />}
            title="Provisioning failed"
          >
            {error}
          </Banner>
        )}
      </div>
    </Drawer>
  );
}

function ProvisionSteps({ steps }: { steps: JobStepState[] }) {
  return (
    <ol className="flex flex-col gap-4">
      {steps.map((step) => (
        <li key={step.id} className="flex items-center gap-3 text-body-sm">
          <StepStatusIcon status={step.status} />
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

function StepStatusIcon({ status }: { status: JobStepStatus }) {
  switch (status) {
    case JobStepStatus.Running:
      return <Sparkles size={16} className="text-fg-brand animate-pulse" />;
    case JobStepStatus.Completed:
      return <Check size={16} className="text-fg-success-moderate" />;
    case JobStepStatus.Failed:
      return <X size={16} className="text-fg-danger-moderate" />;
    case JobStepStatus.Pending:
      return <Circle size={16} className="text-fg-brand" />;
  }
}
