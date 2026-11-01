import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { FormField } from "../primitives/FormField";
import { AccountEditForm } from "./AccountEditForm";
import { AccountRow } from "./AccountRow";

interface AwsAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onVerifyRequested: (provider: CloudProviderName) => void;
}

export function AwsAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onVerifyRequested,
}: AwsAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [formFields, setFormFields] = useState({ accessKey: "", secretKey: "" });

  const { isSaving, error, saveCredentials, deleteCredentials, loadCredentials, clearError } =
    useCredentials();

  useEffect(() => {
    void loadCredentials(CloudProviderName.Aws).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, [loadCredentials]);

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
    const success = await saveCredentials({
      provider: CloudProviderName.Aws,
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

  if (!isEditing) {
    return (
      <AccountRow
        provider={CloudProviderName.Aws}
        title="AWS Account"
        hasCredentials={hasCredentials}
        isConfirmingDelete={isConfirmingDelete}
        onEdit={handleEditOpen}
        onVerify={() => onVerifyRequested(CloudProviderName.Aws)}
        onRequestDelete={() => setIsConfirmingDelete(true)}
        onCancelDelete={() => setIsConfirmingDelete(false)}
        onConfirmDelete={handleDelete}
      />
    );
  }

  return (
    <AccountEditForm
      provider={CloudProviderName.Aws}
      title={hasCredentials ? "Edit AWS Account" : "Add AWS Account"}
      subtitle={
        hasCredentials ? "Update your AWS access credentials" : "Enter your AWS access credentials"
      }
      error={error}
      isSaving={isSaving}
      isFormValid={!!formFields.accessKey.trim()}
      onCancel={handleCancel}
      onSave={handleSave}
    >
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
    </AccountEditForm>
  );
}
