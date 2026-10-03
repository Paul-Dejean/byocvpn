import { Check, Circle, Sparkles, X } from "lucide-react";
import {
  Permissions,
  CloudProviderName,
  JobStepState,
  JobStepStatus,
} from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { Drawer } from "../primitives/Drawer";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";
import { Alert } from "../primitives/Alert";
import { PermissionsPanel } from "./PermissionsPanel";

export interface VerificationState {
  isVerifying: boolean;
  permissions: Permissions | null;
  failed: boolean;
}

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
  verification?: VerificationState;
  onRetry?: () => void;
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
  verification,
  onRetry,
}: JobProgressDrawerProps) {
  const isVerifying = verification?.isVerifying === true;
  const verificationFailed = verification?.failed === true;

  const drawerTitle = verificationFailed
    ? "Permission check failed"
    : isVerifying
      ? "Verifying permissions"
      : (title ?? `Provisioning ${PROVIDER_METADATA[provider].shortLabel}`);

  const drawerSubtitle = verificationFailed
    ? "Your access key is missing required permissions"
    : isVerifying
      ? "Checking your access key against the required permissions"
      : (subtitle ?? "Setting up your account infrastructure");

  return (
    <Drawer
      isOpen={isOpen}
      onClose={onClose}
      title={drawerTitle}
      subtitle={drawerSubtitle}
      footer={
        verificationFailed ? (
          <Button variant="primary" size="none" onClick={onRetry} className="w-full py-2 text-sm">
            Retry
          </Button>
        ) : (
          <Button variant="secondary" size="none" onClick={onClose} className="w-full py-2 text-sm">
            Close
          </Button>
        )
      }
    >
      {isVerifying ? (
        <div className="flex items-center gap-2 text-sm text-gray-300">
          <Spinner color="border-gray-400" />
          Verifying permissions
        </div>
      ) : verificationFailed ? (
        <div className="flex flex-col gap-4">
          <Alert variant="error" title="Update your permissions">
            Your credentials are missing the permissions marked below. Grant
            them in your cloud provider's console, then retry.
          </Alert>
          <PermissionsPanel
            permissions={verification?.permissions ?? null}
            isVerifying={false}
            error={null}
          />
        </div>
      ) : (
        <div className="flex flex-col gap-6">
          <ProvisionSteps steps={steps} />

          {verification?.permissions && (
            <PermissionsPanel
              permissions={verification.permissions}
              isVerifying={false}
              error={null}
            />
          )}

          {isComplete && (
            <Alert
              variant="success"
              icon={<Check size={16} className="text-success-400" />}
              title={successMessage ?? "Account provisioned successfully"}
            />
          )}

          {error && (
            <Alert
              variant="error"
              icon={<X size={16} className="text-danger-400" />}
              title="Provisioning failed"
            >
              {error}
            </Alert>
          )}
        </div>
      )}
    </Drawer>
  );
}

function ProvisionSteps({ steps }: { steps: JobStepState[] }) {
  return (
    <ol className="flex flex-col gap-4">
      {steps.map((step) => (
        <li key={step.id} className="flex items-center gap-3 text-sm">
          <StepStatusIcon status={step.status} />
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

function StepStatusIcon({ status }: { status: JobStepStatus }) {
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
