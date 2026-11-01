import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { FormField } from "../primitives/FormField";
import { AccountEditForm } from "./AccountEditForm";
import { AccountRow } from "./AccountRow";

interface AzureAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onVerifyRequested: (provider: CloudProviderName) => void;
}

export function AzureAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onVerifyRequested,
}: AzureAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [secretAlreadySet, setSecretAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState({
    subscriptionId: "",
    tenantId: "",
    applicationId: "",
    secretValue: "",
  });

  const { isSaving, error, saveCredentials, deleteCredentials, loadCredentials, clearError } =
    useCredentials();

  useEffect(() => {
    void loadCredentials(CloudProviderName.Azure).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, [loadCredentials]);

  const resetForm = () => {
    setFormFields({ subscriptionId: "", tenantId: "", applicationId: "", secretValue: "" });
    setSecretAlreadySet(false);
  };

  const handleEditOpen = async () => {
    const existing = await loadCredentials(CloudProviderName.Azure);
    if (existing) {
      setFormFields({
        subscriptionId: existing.subscriptionId,
        tenantId: existing.tenantId,
        applicationId: existing.applicationId,
        secretValue: "",
      });
      setSecretAlreadySet(!!existing.secretValue);
    }
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    resetForm();
    clearError();
  };

  const handleSave = async () => {
    const success = await saveCredentials({
      provider: CloudProviderName.Azure,
      subscriptionId: formFields.subscriptionId.trim(),
      tenantId: formFields.tenantId.trim(),
      applicationId: formFields.applicationId.trim(),
      secretValue: formFields.secretValue.trim(),
    });

    if (success) {
      resetForm();
      setIsEditing(false);
      setHasCredentials(true);
      onCredentialsSaved(CloudProviderName.Azure);
    }
  };

  const handleDeleteCredentials = async () => {
    const success = await deleteCredentials(CloudProviderName.Azure);
    if (success) {
      setHasCredentials(false);
      setIsConfirmingDelete(false);
      onCredentialsDeleted();
    }
  };

  const isFormValid =
    formFields.subscriptionId.trim() &&
    formFields.tenantId.trim() &&
    formFields.applicationId.trim() &&
    (formFields.secretValue.trim() || secretAlreadySet);

  if (!isEditing) {
    return (
      <AccountRow
        provider={CloudProviderName.Azure}
        title="Azure Account"
        hasCredentials={hasCredentials}
        isConfirmingDelete={isConfirmingDelete}
        onEdit={handleEditOpen}
        onVerify={() => onVerifyRequested(CloudProviderName.Azure)}
        onRequestDelete={() => setIsConfirmingDelete(true)}
        onCancelDelete={() => setIsConfirmingDelete(false)}
        onConfirmDelete={handleDeleteCredentials}
      />
    );
  }

  return (
    <AccountEditForm
      provider={CloudProviderName.Azure}
      title={hasCredentials ? "Edit Azure Account" : "Add Azure Account"}
      subtitle={
        hasCredentials
          ? "Update your Azure service-principal credentials"
          : "Enter your Azure service-principal credentials"
      }
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onCancel={handleCancel}
      onSave={handleSave}
    >
      <FormField
        label="Subscription ID"
        hint="e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
        type="text"
        mono
        value={formFields.subscriptionId}
        onChange={(value) => setFormFields((prev) => ({ ...prev, subscriptionId: value }))}
        placeholder="00000000-0000-0000-0000-000000000000"
      />

      <FormField
        label="Tenant ID"
        hint="Your Azure Active Directory tenant ID (UUID)"
        type="text"
        mono
        value={formFields.tenantId}
        onChange={(value) => setFormFields((prev) => ({ ...prev, tenantId: value }))}
        placeholder="00000000-0000-0000-0000-000000000000"
      />

      <FormField
        label="Application ID"
        hint="Application ID of the service principal"
        type="text"
        mono
        value={formFields.applicationId}
        onChange={(value) => setFormFields((prev) => ({ ...prev, applicationId: value }))}
        placeholder="00000000-0000-0000-0000-000000000000"
      />

      <FormField
        label="Secret Value"
        hint={
          secretAlreadySet
            ? "Secret value already configured. Enter a new value to replace it."
            : "Secret value from your app registration"
        }
        type="password"
        mono
        value={formFields.secretValue}
        onChange={(value) => setFormFields((prev) => ({ ...prev, secretValue: value }))}
        placeholder={secretAlreadySet ? "Enter new secret to replace" : "Secret value"}
      />
    </AccountEditForm>
  );
}
