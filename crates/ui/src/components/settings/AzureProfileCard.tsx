import { Spinner } from "../common/Spinner";
import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";

interface AzureProfileCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}

function AzureIcon() {
  return (
    <div className="w-12 h-12 bg-white/5 rounded-lg flex items-center justify-center flex-shrink-0 p-2">
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
    clientId: "",
    clientSecret: "",
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
    setFormFields({ subscriptionId: "", tenantId: "", clientId: "", clientSecret: "" });
    setSecretAlreadySet(false);
  };

  const handleEditOpen = async () => {
    const existing = await loadCredentials(CloudProviderName.Azure);
    if (existing) {
      setFormFields({
        subscriptionId: existing.subscriptionId,
        tenantId: existing.tenantId,
        clientId: existing.clientId,
        clientSecret: "",
      });
      setSecretAlreadySet(!!existing.clientSecret);
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
      clientId: formFields.clientId.trim(),
      clientSecret: formFields.clientSecret.trim(),
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
    formFields.clientId.trim() &&
    (formFields.clientSecret.trim() || secretAlreadySet);

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div
      className={`rounded-lg p-6 mt-4 ${
        showNotProvisionedWarning
          ? "bg-gray-700 border-l-4 border-amber-500"
          : "bg-gray-700"
      }`}
    >
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <AzureIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-white">
                  Azure Profile
                </h3>
                {hasCredentials && isProvisioned && (
                  <span className="text-xs px-2 py-0.5 bg-green-900/50 text-green-400 rounded-full border border-green-700/50">
                    Provisioned
                  </span>
                )}
                {showNotProvisionedWarning && (
                  <span className="text-xs px-2 py-0.5 bg-yellow-900/50 text-yellow-300 rounded-full border border-yellow-700/50">
                    Not provisioned
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-400">
                Azure service-principal credentials for VM deployment
              </p>
            </div>
          </div>

          {hasCredentials === null ? (
            <Spinner color="border-gray-400" />
          ) : hasCredentials ? (
            <div className="flex items-center gap-2">
              {isConfirmingDelete ? (
                <>
                  <span className="text-sm text-gray-300">Delete?</span>
                  <button
                    onClick={() => setIsConfirmingDelete(false)}
                    className="px-3 py-1.5 btn-secondary text-sm"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleDeleteCredentials}
                    className="px-3 py-1.5 btn-danger text-sm"
                  >
                    Confirm
                  </button>
                </>
              ) : (
                <>
                  {isProvisioned ? (
                    <button
                      onClick={() => onProvisionRequested(CloudProviderName.Azure)}
                      className="p-2 text-gray-400 hover:text-blue-400 hover:bg-gray-600 rounded-lg transition-colors"
                      title="Re-provision"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </button>
                  ) : (
                    <button
                      onClick={() => onProvisionRequested(CloudProviderName.Azure)}
                      className="p-2 text-amber-400 hover:text-amber-300 hover:bg-gray-600 rounded-lg transition-colors"
                      title="Provision"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                      </svg>
                    </button>
                  )}
                  <button
                    onClick={() => setIsConfirmingDelete(true)}
                    className="p-2 text-gray-400 hover:text-red-400 hover:bg-gray-600 rounded-lg transition-colors"
                    title="Delete credentials"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  <button
                    onClick={handleEditOpen}
                    className="px-4 py-2 btn-primary font-medium flex items-center gap-2"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Edit
                  </button>
                </>
              )}
            </div>
          ) : (
            <button
              onClick={handleEditOpen}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors font-medium flex items-center gap-2"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Add Provider
            </button>
          )}
        </div>
      ) : (
        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <AzureIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                {hasCredentials ? "Edit Azure Profile" : "Add Azure Profile"}
              </h3>
              <p className="text-sm text-gray-400">
                {hasCredentials
                  ? "Update your Azure service-principal credentials"
                  : "Enter your Azure service-principal credentials"}
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Subscription ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
              </p>
              <input
                type="text"
                value={formFields.subscriptionId}
                onChange={(e) => setFormFields((prev) => ({ ...prev, subscriptionId: e.target.value }))}
                className="input font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Tenant ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                Your Azure Active Directory tenant ID (UUID)
              </p>
              <input
                type="text"
                value={formFields.tenantId}
                onChange={(e) => setFormFields((prev) => ({ ...prev, tenantId: e.target.value }))}
                className="input font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Client ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                Application (client) ID of the service principal
              </p>
              <input
                type="text"
                value={formFields.clientId}
                onChange={(e) => setFormFields((prev) => ({ ...prev, clientId: e.target.value }))}
                className="input font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Client Secret
              </label>
              {secretAlreadySet && !formFields.clientSecret && (
                <p className="text-xs text-green-400 mb-2">
                  ✓ Client secret already configured — enter a new value to
                  replace it
                </p>
              )}
              {!secretAlreadySet && !formFields.clientSecret && (
                <p className="text-xs text-gray-500 mb-2">
                  Secret value from your app registration
                </p>
              )}
              <input
                type="password"
                value={formFields.clientSecret}
                onChange={(e) => setFormFields((prev) => ({ ...prev, clientSecret: e.target.value }))}
                className="input font-mono text-sm"
                placeholder={secretAlreadySet ? "Enter new secret to replace" : ""}
              />
            </div>

            {error && (
              <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                <p className="text-red-300 text-sm">{error}</p>
              </div>
            )}

            {successMessage && (
              <div className="p-3 bg-green-900 border border-green-700 rounded-lg">
                <p className="text-green-300 text-sm">{successMessage}</p>
              </div>
            )}

            <div className="flex gap-3 pt-4">
              <button
                onClick={handleCancel}
                className="flex-1 px-4 py-2 btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                disabled={isSaving || !isFormValid}
                className="btn-primary flex-1 px-4 py-2 disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-600"
              >
                {isSaving ? (
                  <div className="flex items-center justify-center gap-2">
                    <Spinner color="border-white" />
                    Saving...
                  </div>
                ) : (
                  "Save Profile"
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
