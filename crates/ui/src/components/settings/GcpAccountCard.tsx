import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { readProjectIdFromServiceAccountJson } from "../../lib/serviceAccountJson";
import { KeyFileField } from "../accounts/KeyFileField";
import { FormField } from "../primitives/FormField";
import { AccountEditForm } from "./AccountEditForm";
import { AccountRow } from "./AccountRow";

interface GcpAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onVerifyRequested: (provider: CloudProviderName) => void;
}

const EMPTY_FORM = { projectId: "", serviceAccountJson: "" };

export function GcpAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onVerifyRequested,
}: GcpAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [jsonAlreadySet, setJsonAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState(EMPTY_FORM);

  const { isSaving, error, saveCredentials, deleteCredentials, loadCredentials, clearError } =
    useCredentials();

  useEffect(() => {
    void loadCredentials(CloudProviderName.Gcp).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, [loadCredentials]);

  const resetForm = () => {
    setFormFields(EMPTY_FORM);
    setJsonAlreadySet(false);
  };

  const handleEditOpen = async () => {
    const existing = await loadCredentials(CloudProviderName.Gcp);
    if (existing) {
      setFormFields({ projectId: existing.projectId, serviceAccountJson: "" });
      setJsonAlreadySet(!!existing.serviceAccountJson);
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
      provider: CloudProviderName.Gcp,
      projectId: formFields.projectId.trim(),
      serviceAccountJson: formFields.serviceAccountJson.trim(),
    });

    if (success) {
      resetForm();
      setIsEditing(false);
      setHasCredentials(true);
      onCredentialsSaved(CloudProviderName.Gcp);
    }
  };

  const handleKeyFileLoaded = (content: string) => {
    const parsedProjectId = readProjectIdFromServiceAccountJson(content);
    if (parsedProjectId) {
      setFormFields((prev) => ({ ...prev, projectId: prev.projectId || parsedProjectId }));
    }
  };

  const handleDeleteCredentials = async () => {
    const success = await deleteCredentials(CloudProviderName.Gcp);
    if (success) {
      setHasCredentials(false);
      setIsConfirmingDelete(false);
      onCredentialsDeleted();
    }
  };

  const isFormValid =
    formFields.projectId.trim() && (formFields.serviceAccountJson.trim() || jsonAlreadySet);

  if (!isEditing) {
    return (
      <AccountRow
        provider={CloudProviderName.Gcp}
        title="Google Cloud Account"
        hasCredentials={hasCredentials}
        isConfirmingDelete={isConfirmingDelete}
        onEdit={handleEditOpen}
        onVerify={() => onVerifyRequested(CloudProviderName.Gcp)}
        onRequestDelete={() => setIsConfirmingDelete(true)}
        onCancelDelete={() => setIsConfirmingDelete(false)}
        onConfirmDelete={handleDeleteCredentials}
      />
    );
  }

  return (
    <AccountEditForm
      provider={CloudProviderName.Gcp}
      title={hasCredentials ? "Edit Google Cloud Account" : "Add Google Cloud Account"}
      subtitle={
        hasCredentials
          ? "Update your GCP service-account key"
          : "Enter your GCP service-account key"
      }
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onCancel={handleCancel}
      onSave={handleSave}
    >
      <FormField
        label="Project ID"
        hint="e.g. my-project-123456"
        type="text"
        mono
        value={formFields.projectId}
        onChange={(value) => setFormFields((prev) => ({ ...prev, projectId: value }))}
        placeholder="my-gcp-project"
      />
      <KeyFileField
        label="Service Account Key (.json)"
        accept=".json,application/json"
        value={formFields.serviceAccountJson}
        onChange={(content) => setFormFields((prev) => ({ ...prev, serviceAccountJson: content }))}
        onFileLoaded={handleKeyFileLoaded}
        placeholder='{"type":"service_account","project_id":"..."}'
        alreadySetMessage={
          jsonAlreadySet
            ? "✓ Service account key already configured — load a new file or paste below to replace it"
            : undefined
        }
        emptyHint='Paste the contents of your service-account JSON key file or use "Load from file"'
      />
    </AccountEditForm>
  );
}
