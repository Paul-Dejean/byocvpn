import { ReactNode } from "react";
import { AppError, CloudProviderName } from "../../types";
import { Button } from "../primitives/Button";
import { ErrorBanner } from "../common/ErrorBanner";
import { AccountIconTile } from "./AccountRow";

interface AccountEditFormProps {
  provider: CloudProviderName;
  title: string;
  subtitle: string;
  error: AppError | null;
  isSaving: boolean;
  isFormValid: boolean;
  onCancel: () => void;
  onSave: () => void;
  children: ReactNode;
}

export function AccountEditForm({
  provider,
  title,
  subtitle,
  error,
  isSaving,
  isFormValid,
  onCancel,
  onSave,
  children,
}: AccountEditFormProps) {
  return (
    <div className="flex flex-col gap-5 p-4">
      <div className="flex items-center gap-3">
        <AccountIconTile provider={provider} />
        <div>
          <h3 className="text-body-sm text-fg-lighter">{title}</h3>
          <p className="text-caption text-fg-medium">{subtitle}</p>
        </div>
      </div>

      <div className="flex flex-col gap-4">
        {children}

        {error && <ErrorBanner error={error} />}

        <div className="flex gap-3 pt-2">
          <Button variant="secondary" size="lg" onClick={onCancel} className="flex-1">
            Cancel
          </Button>
          <Button
            variant="primary"
            size="lg"
            onClick={onSave}
            loading={isSaving}
            disabled={!isFormValid}
            className="flex-1"
          >
            {isSaving ? "Saving..." : "Save Account"}
          </Button>
        </div>
      </div>
    </div>
  );
}
