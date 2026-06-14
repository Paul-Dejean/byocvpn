import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Badge } from "../primitives/Badge";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { Alert } from "../primitives/Alert";
import { FormField } from "../primitives/FormField";

interface AzureProfileCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}

function AzureIcon() {
  return (
    <div className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 p-2.5">
      <img src="/cloud-providers/azure-icon.svg" alt="Azure" className="w-full h-full object-contain" />
    </div>
  );
}

export function AzureProfileCard({
  onCredentialsSaved,
  onCredentialsDeleted,
  onProvisionRequested,
  isProvisioned,
}: AzureProfileCardProps) {
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
    successMessage,
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

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div className="py-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <AzureIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-primary">
                  Azure Account
                </h3>
                {hasCredentials && isProvisioned && (
                  <Badge variant="success" shape="pill">
                    Provisioned
                  </Badge>
                )}
                {showNotProvisionedWarning && (
                  <Badge variant="warning" shape="pill">
                    Not provisioned
                  </Badge>
                )}
              </div>
            </div>
          </div>

          {hasCredentials === null ? (
            <Spinner color="border-gray-400" />
          ) : hasCredentials ? (
            <div className="flex items-center gap-2">
              {isConfirmingDelete ? (
                <>
                  <span className="text-sm text-gray-300">Delete?</span>
                  <Button variant="secondary" size="sm" onClick={() => setIsConfirmingDelete(false)}>Cancel</Button>
                  <Button variant="danger" size="sm" onClick={handleDeleteCredentials}>Confirm</Button>
                </>
              ) : (
                <>
                  {isProvisioned ? (
                    <IconButton accent="blue" onClick={() => onProvisionRequested(CloudProviderName.Azure)} title="Re-provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </IconButton>
                  ) : (
                    <IconButton accent="amber" onClick={() => onProvisionRequested(CloudProviderName.Azure)} title="Provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                      </svg>
                    </IconButton>
                  )}
                  <IconButton accent="red" onClick={() => setIsConfirmingDelete(true)} title="Delete credentials">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </IconButton>
                  <Button
                    variant="primary"
                    onClick={handleEditOpen}
                    icon={
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    }
                  >
                    Edit
                  </Button>
                </>
              )}
            </div>
          ) : (
            <Button
              variant="success"
              onClick={handleEditOpen}
              icon={
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              }
            >
              Add Provider
            </Button>
          )}
        </div>
      ) : (
        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <AzureIcon />
            <div>
              <h3 className="font-semibold text-lg text-primary">
                {hasCredentials ? "Edit Azure Account" : "Add Azure Account"}
              </h3>
              <p className="text-sm text-gray-400">
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
            {successMessage && <Alert variant="success">{successMessage}</Alert>}

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
