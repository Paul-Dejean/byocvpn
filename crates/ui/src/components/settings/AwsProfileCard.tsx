import { Spinner } from "../common/Spinner";
import { useEffect, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";

interface AwsProfileCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}

function AwsIcon() {
  return (
    <div className="w-12 h-12 rounded-xl flex items-center justify-center p-2.5 flex-shrink-0">
      <img src="/cloud-providers/aws-icon.svg" alt="AWS" className="w-full h-full object-contain" />
    </div>
  );
}

export function AwsProfileCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: AwsProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [formFields, setFormFields] = useState({ accessKey: "", secretKey: "" });

  const { isSaving, error, successMessage, saveCredentials, deleteCredentials, loadCredentials, clearError } = useCredentials();

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

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div className="py-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <AwsIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-white">AWS Account</h3>
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
            </div>
          </div>

          {hasCredentials === null ? (
            <Spinner color="border-gray-400" />
          ) : hasCredentials ? (
            <div className="flex items-center gap-2">
              {isConfirmingDelete ? (
                <>
                  <span className="text-sm text-gray-300">Delete?</span>
                  <button onClick={() => setIsConfirmingDelete(false)} className="px-3 py-1.5 btn-secondary text-sm">Cancel</button>
                  <button onClick={handleDelete} className="px-3 py-1.5 btn-danger text-sm">Confirm</button>
                </>
              ) : (
                <>
                  {isProvisioned ? (
                    <button onClick={() => onProvisionRequested(CloudProviderName.Aws)} className="p-2 text-gray-400 hover:text-blue-400 hover:bg-gray-600 rounded-lg transition-colors" title="Re-provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </button>
                  ) : (
                    <button onClick={() => onProvisionRequested(CloudProviderName.Aws)} className="p-2 text-amber-400 hover:text-amber-300 hover:bg-gray-600 rounded-lg transition-colors" title="Provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                      </svg>
                    </button>
                  )}
                  <button onClick={() => setIsConfirmingDelete(true)} className="p-2 text-gray-400 hover:text-red-400 hover:bg-gray-600 rounded-lg transition-colors" title="Delete credentials">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  <button onClick={handleEditOpen} className="px-4 py-2 btn-primary font-medium flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Edit
                  </button>
                </>
              )}
            </div>
          ) : (
            <button onClick={handleEditOpen} className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors font-medium flex items-center gap-2">
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
            <AwsIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">{hasCredentials ? "Edit AWS Account" : "Add AWS Account"}</h3>
              <p className="text-sm text-gray-400">{hasCredentials ? "Update your AWS access credentials" : "Enter your AWS access credentials"}</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Access Key ID</label>
              <p className="text-xs text-gray-500 mb-2">e.g. AKIAIOSFODNN7EXAMPLE</p>
              <input
                type="text"
                value={formFields.accessKey}
                onChange={(e) => setFormFields((prev) => ({ ...prev, accessKey: e.target.value }))}
                className="input font-mono text-sm"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Secret Access Key</label>
              <p className="text-xs text-gray-500 mb-2">Leave blank to keep your existing key</p>
              <input
                type="password"
                value={formFields.secretKey}
                onChange={(e) => setFormFields((prev) => ({ ...prev, secretKey: e.target.value }))}
                className="input font-mono text-sm"
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
              <button onClick={handleCancel} className="flex-1 px-4 py-2 btn-secondary">Cancel</button>
              <button
                onClick={handleSave}
                disabled={isSaving || !formFields.accessKey.trim()}
                className="btn-primary flex-1 px-4 py-2 disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-600"
              >
                {isSaving ? (
                  <div className="flex items-center justify-center gap-2">
                    <Spinner color="border-white" />
                    Saving...
                  </div>
                ) : (
                  "Save Account"
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
