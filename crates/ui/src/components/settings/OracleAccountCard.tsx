import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { KeyFileField } from "../accounts/KeyFileField";
import { FormField } from "../primitives/FormField";
import { AccountEditForm } from "./AccountEditForm";
import { AccountRow } from "./AccountRow";

interface OracleAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
}

const EMPTY_FORM = {
  tenancyOcid: "",
  userOcid: "",
  fingerprint: "",
  privateKeyPem: "",
  region: "",
};

export function OracleAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
}: OracleAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [pemAlreadySet, setPemAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState(EMPTY_FORM);

  const { isSaving, error, saveCredentials, deleteCredentials, loadCredentials, clearError } =
    useCredentials();

  useEffect(() => {
    void loadCredentials(CloudProviderName.Oracle).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, [loadCredentials]);

  const resetForm = () => {
    setFormFields(EMPTY_FORM);
    setPemAlreadySet(false);
  };

  const handleEditOpen = async () => {
    const existing = await loadCredentials(CloudProviderName.Oracle);
    if (existing) {
      setFormFields({
        tenancyOcid: existing.tenancyOcid,
        userOcid: existing.userOcid,
        fingerprint: existing.fingerprint,
        region: existing.region,
        privateKeyPem: existing.privateKeyPem ?? "",
      });
      setPemAlreadySet(!!existing.privateKeyPem);
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
      provider: CloudProviderName.Oracle,
      tenancyOcid: formFields.tenancyOcid.trim(),
      userOcid: formFields.userOcid.trim(),
      fingerprint: formFields.fingerprint.trim(),
      privateKeyPem: formFields.privateKeyPem.trim(),
      region: formFields.region.trim(),
    });

    if (success) {
      resetForm();
      setIsEditing(false);
      setHasCredentials(true);
      onCredentialsSaved(CloudProviderName.Oracle);
    }
  };

  const handleDeleteCredentials = async () => {
    const success = await deleteCredentials(CloudProviderName.Oracle);
    if (success) {
      setHasCredentials(false);
      setIsConfirmingDelete(false);
      onCredentialsDeleted();
    }
  };

  const isFormValid =
    formFields.tenancyOcid.trim() &&
    formFields.userOcid.trim() &&
    formFields.fingerprint.trim() &&
    (formFields.privateKeyPem.trim() || pemAlreadySet) &&
    formFields.region.trim();

  if (!isEditing) {
    return (
      <AccountRow
        provider={CloudProviderName.Oracle}
        title="Oracle Cloud Account"
        hasCredentials={hasCredentials}
        isConfirmingDelete={isConfirmingDelete}
        onEdit={handleEditOpen}
        onRequestDelete={() => setIsConfirmingDelete(true)}
        onCancelDelete={() => setIsConfirmingDelete(false)}
        onConfirmDelete={handleDeleteCredentials}
      />
    );
  }

  return (
    <AccountEditForm
      provider={CloudProviderName.Oracle}
      title={hasCredentials ? "Edit Oracle Cloud Account" : "Add Oracle Cloud Account"}
      subtitle={
        hasCredentials
          ? "Update your OCI API signing credentials"
          : "Enter your OCI API signing credentials"
      }
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onCancel={handleCancel}
      onSave={handleSave}
    >
      <FormField
        label="Tenancy OCID"
        hint="e.g. ocid1.tenancy.oc1..aaaaaa…"
        type="text"
        mono
        value={formFields.tenancyOcid}
        onChange={(value) => setFormFields((prev) => ({ ...prev, tenancyOcid: value }))}
      />
      <FormField
        label="User OCID"
        hint="e.g. ocid1.user.oc1..aaaaaa…"
        type="text"
        mono
        value={formFields.userOcid}
        onChange={(value) => setFormFields((prev) => ({ ...prev, userOcid: value }))}
      />
      <FormField
        label="Key Fingerprint"
        hint="e.g. xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx"
        type="text"
        mono
        value={formFields.fingerprint}
        onChange={(value) => setFormFields((prev) => ({ ...prev, fingerprint: value }))}
      />
      <FormField
        label="Home Region"
        hint="e.g. us-ashburn-1"
        type="text"
        mono
        value={formFields.region}
        onChange={(value) => setFormFields((prev) => ({ ...prev, region: value }))}
      />
      <KeyFileField
        label="Private Key (.pem)"
        accept=".pem"
        value={formFields.privateKeyPem}
        onChange={(content) => setFormFields((prev) => ({ ...prev, privateKeyPem: content }))}
        alreadySetMessage={
          pemAlreadySet
            ? "✓ Private key already configured — load a new file or paste below to replace it"
            : undefined
        }
        emptyHint='Paste the contents of your .pem file or use "Load from file"'
      />
    </AccountEditForm>
  );
}
