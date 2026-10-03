import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Button } from "../primitives/Button";
import { AccountIconTile, AccountRow } from "./AccountRow";
import { Alert } from "../primitives/Alert";
import { FormField } from "../primitives/FormField";

interface AzureAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}


export function AzureAccountCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onProvisionRequested,
  isProvisioned,
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

  const {
    isSaving,
    error,
    saveCredentials,
    deleteCredentials,
    loadCredentials,
    clearError,
  } = useCredentials();

  useEffect(() => {
    loadCredentials(CloudProviderName.Azure).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, []);

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
    const success = await saveCredentials(CloudProviderName.Azure, {
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

  return (
    <div>
      {!isEditing ? (
        <AccountRow
          provider={CloudProviderName.Azure}
          title="Azure Account"
          hasCredentials={hasCredentials}
          isProvisioned={isProvisioned}
          isConfirmingDelete={isConfirmingDelete}
          onEdit={handleEditOpen}
          onProvision={() => onProvisionRequested(CloudProviderName.Azure)}
          onRequestDelete={() => setIsConfirmingDelete(true)}
          onCancelDelete={() => setIsConfirmingDelete(false)}
          onConfirmDelete={handleDeleteCredentials}
        />
      ) : (
        <div className="p-4 flex flex-col gap-5">
          <div className="flex items-center gap-3">
            <AccountIconTile provider={CloudProviderName.Azure} />
            <div>
              <h3 className="text-sm text-primary">
                {hasCredentials ? "Edit Azure Account" : "Add Azure Account"}
              </h3>
              <p className="text-xs text-gray-300">
                {hasCredentials
                  ? "Update your Azure service-principal credentials"
                  : "Enter your Azure service-principal credentials"}
              </p>
            </div>
          </div>

          <div className="space-y-4">
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

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Secret Value
              </label>
              {secretAlreadySet && !formFields.secretValue && (
                <p className="text-xs text-success-400 mb-2">
                  ✓ Secret value already configured — enter a new value to
                  replace it
                </p>
              )}
              {!secretAlreadySet && !formFields.secretValue && (
                <p className="text-xs text-gray-500 mb-2">
                  Secret value from your app registration
                </p>
              )}
              <input
                type="password"
                value={formFields.secretValue}
                onChange={(e) => setFormFields((prev) => ({ ...prev, secretValue: e.target.value }))}
                className="input font-mono text-sm"
                placeholder={secretAlreadySet ? "Enter new secret to replace" : ""}
              />
            </div>

            {error && <Alert variant="error">{error}</Alert>}

            <div className="flex gap-3 pt-4">
              <Button variant="secondary" onClick={handleCancel} className="flex-1">Cancel</Button>
              <Button
                variant="primary"
                onClick={handleSave}
                loading={isSaving}
                disabled={!isFormValid}
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
