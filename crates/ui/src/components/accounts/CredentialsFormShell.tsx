import { ReactNode } from "react";
import { AppError } from "../../types";
import { Button } from "../primitives/Button";
import { ErrorBanner } from "../common/ErrorBanner";

interface CredentialsFormShellProps {
  error: AppError | null;
  isSaving: boolean;
  isFormValid: boolean;
  onSubmit: () => void;
  onCancel: () => void;
  children: ReactNode;
}

export function CredentialsFormShell({
  error,
  isSaving,
  isFormValid,
  onSubmit,
  onCancel,
  children,
}: CredentialsFormShellProps) {
  return (
    <div className="flex flex-col gap-4">
      {children}

      {error && <ErrorBanner error={error} />}

      <div className="flex gap-3 pt-1">
        <Button variant="secondary" size="lg" onClick={onCancel} className="flex-1">
          Back
        </Button>
        <Button
          variant="primary"
          size="lg"
          onClick={onSubmit}
          loading={isSaving}
          disabled={!isFormValid}
          disabledStyle="dim"
          className="flex-1"
        >
          {isSaving ? "Saving" : "Connect"}
        </Button>
      </div>
    </div>
  );
}
