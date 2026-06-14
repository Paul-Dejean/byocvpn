import { useEffect, useRef, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Badge } from "../primitives/Badge";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { Alert } from "../primitives/Alert";
import { FormField } from "../primitives/FormField";

interface GcpProfileCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}

function GcpIcon() {
  return (
    <div className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 p-2.5">
      <img src="/cloud-providers/google-cloud-icon.svg" alt="GCP" className="w-full h-full object-contain" />
    </div>
  );
}

export function GcpProfileCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: GcpProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [jsonAlreadySet, setJsonAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState({ projectId: "", serviceAccountJson: "" });

  const fileInputRef = useRef<HTMLInputElement>(null);

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
    loadCredentials(CloudProviderName.Gcp).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, []);

  const resetForm = () => {
    setFormFields({ projectId: "", serviceAccountJson: "" });
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
    const success = await saveCredentials(CloudProviderName.Gcp, {
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

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const content = loadEvent.target?.result;
      if (typeof content === "string") {
        try {
          const parsed = JSON.parse(content);
          setFormFields((prev) => ({
            serviceAccountJson: content,
            projectId: parsed.project_id && !prev.projectId ? parsed.project_id : prev.projectId,
          }));
        } catch {
          setFormFields((prev) => ({ ...prev, serviceAccountJson: content }));
        }
      }
    };
    reader.readAsText(file);
    event.target.value = "";
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

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div className="py-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <GcpIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-primary">Google Cloud Account</h3>
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
                    <IconButton accent="blue" onClick={() => onProvisionRequested(CloudProviderName.Gcp)} title="Re-provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </IconButton>
                  ) : (
                    <IconButton accent="amber" onClick={() => onProvisionRequested(CloudProviderName.Gcp)} title="Provision">
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
            <GcpIcon />
            <div>
              <h3 className="font-semibold text-lg text-primary">
                {hasCredentials
                  ? "Edit Google Cloud Account"
                  : "Add Google Cloud Account"}
              </h3>
              <p className="text-sm text-gray-400">
                {hasCredentials
                  ? "Update your GCP service-account key"
                  : "Enter your GCP service-account key"}
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <FormField
              label="Project ID"
              hint="e.g. my-project-123456"
              type="text"
              mono
              value={formFields.projectId}
              onChange={(value) => setFormFields((prev) => ({ ...prev, projectId: value }))}
              placeholder="my-gcp-project"
            />

            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="block text-sm font-medium text-gray-300">
                  Service Account Key (.json)
                </label>
                <Button
                  variant="secondary"
                  size="none"
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  className="text-xs px-3 py-1"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-3.5 w-3.5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
                    />
                  </svg>
                  Load from file
                </Button>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".json,application/json"
                  onChange={handleFileChange}
                  className="hidden"
                />
              </div>
              {jsonAlreadySet && !formFields.serviceAccountJson && (
                <p className="text-xs text-success-400 mb-2">
                  ✓ Service account key already configured — load a new file or
                  paste below to replace it
                </p>
              )}
              {!jsonAlreadySet && !formFields.serviceAccountJson && (
                <p className="text-xs text-gray-500 mb-2">
                  Paste the contents of your service-account JSON key file or
                  use "Load from file"
                </p>
              )}
              <textarea
                value={formFields.serviceAccountJson}
                onChange={(e) => setFormFields((prev) => ({ ...prev, serviceAccountJson: e.target.value }))}
                rows={6}
                className="input font-mono text-xs resize-none"
                placeholder='{"type":"service_account","project_id":"..."}'
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
