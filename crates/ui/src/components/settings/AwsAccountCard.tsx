import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Button } from "../primitives/Button";
import { AccountIconTile, AccountRow } from "./AccountRow";
import { Banner } from "../primitives/Banner";
import { FormField } from "../primitives/FormField";

interface AwsAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  onVerifyRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}


export function AwsAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onProvisionRequested,
  onVerifyRequested,
  isProvisioned,
}: AwsAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [formFields, setFormFields] = useState({ accessKey: "", secretKey: "" });

  const { isSaving, error, saveCredentials, deleteCredentials, loadCredentials, clearError } = useCredentials();

  useEffect(() => {
    loadCredentials(CloudProviderName.Aws).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, []);

  const resetForm = () => {
    setFormFields({ accessKey: "", secretKey: "" });
  };

  const handleEditOpen = async () => {
    const existing = await loadCredentials(CloudProviderName.Aws);
    if (existing) {
      setFormFields({ accessKey: existing.accessKeyId, secretKey: "" });
    }
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    resetForm();
    clearError();
  };

  const handleSave = async () => {
    if (!formFields.accessKey.trim()) return;
    const success = await saveCredentials(CloudProviderName.Aws, {
      accessKeyId: formFields.accessKey.trim(),
      secretAccessKey: formFields.secretKey.trim(),
    });
    if (success) {
      resetForm();
      setIsEditing(false);
      setHasCredentials(true);
      onCredentialsSaved(CloudProviderName.Aws);
    }
  };

  const handleDelete = async () => {
    const success = await deleteCredentials(CloudProviderName.Aws);
    if (success) {
      setHasCredentials(false);
      setIsConfirmingDelete(false);
      setIsEditing(false);
      onCredentialsDeleted();
    }
  };

  return (
    <div>
      {!isEditing ? (
        <AccountRow
          provider={CloudProviderName.Aws}
          title="AWS Account"
          hasCredentials={hasCredentials}
          isProvisioned={isProvisioned}
          isConfirmingDelete={isConfirmingDelete}
          onEdit={handleEditOpen}
          onProvision={() => onProvisionRequested(CloudProviderName.Aws)}
          onVerify={() => onVerifyRequested(CloudProviderName.Aws)}
          onRequestDelete={() => setIsConfirmingDelete(true)}
          onCancelDelete={() => setIsConfirmingDelete(false)}
          onConfirmDelete={handleDelete}
        />
      ) : (
        <div className="p-4 flex flex-col gap-5">
          <div className="flex items-center gap-3">
            <AccountIconTile provider={CloudProviderName.Aws} />
            <div>
              <h3 className="text-body-sm text-fg-lighter">{hasCredentials ? "Edit AWS Account" : "Add AWS Account"}</h3>
              <p className="text-caption text-fg-medium">{hasCredentials ? "Update your AWS access credentials" : "Enter your AWS access credentials"}</p>
            </div>
          </div>

          <div className="flex flex-col gap-4">
            <FormField
              label="Access Key ID"
              hint="e.g. AKIAIOSFODNN7EXAMPLE"
              type="text"
              mono
              value={formFields.accessKey}
              onChange={(value) => setFormFields((prev) => ({ ...prev, accessKey: value }))}
            />
            <FormField
              label="Secret Access Key"
              hint="Leave blank to keep your existing key"
              type="password"
              mono
              value={formFields.secretKey}
              onChange={(value) => setFormFields((prev) => ({ ...prev, secretKey: value }))}
            />

            {error && <Banner variant="danger">{error}</Banner>}

            <div className="flex gap-3 pt-2">
              <Button variant="secondary" size="lg" onClick={handleCancel} className="flex-1">Cancel</Button>
              <Button
                variant="primary"
                size="lg"
                onClick={handleSave}
                loading={isSaving}
                disabled={!formFields.accessKey.trim()}
                className="flex-1"
              >
                {isSaving ? "Saving..." : "Save Account"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
