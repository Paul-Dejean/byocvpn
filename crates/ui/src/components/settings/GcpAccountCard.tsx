import { useEffect, useRef, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Upload } from "lucide-react";
import { Button } from "../primitives/Button";
import { AccountIconTile, AccountRow } from "./AccountRow";
import { Banner } from "../primitives/Banner";
import { FormField } from "../primitives/FormField";

interface GcpAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}


export function GcpAccountCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: GcpAccountCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [jsonAlreadySet, setJsonAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState({ projectId: "", serviceAccountJson: "" });

  const fileInputRef = useRef<HTMLInputElement>(null);

  const {
    isSaving,
    error,
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

  return (
    <div>
      {!isEditing ? (
        <AccountRow
          provider={CloudProviderName.Gcp}
          title="Google Cloud Account"
          hasCredentials={hasCredentials}
          isProvisioned={isProvisioned}
          isConfirmingDelete={isConfirmingDelete}
          onEdit={handleEditOpen}
          onProvision={() => onProvisionRequested(CloudProviderName.Gcp)}
          onRequestDelete={() => setIsConfirmingDelete(true)}
          onCancelDelete={() => setIsConfirmingDelete(false)}
          onConfirmDelete={handleDeleteCredentials}
        />
      ) : (
        <div className="p-4 flex flex-col gap-5">
          <div className="flex items-center gap-3">
            <AccountIconTile provider={CloudProviderName.Gcp} />
            <div>
              <h3 className="text-body-sm text-fg-lighter">
                {hasCredentials
                  ? "Edit Google Cloud Account"
                  : "Add Google Cloud Account"}
              </h3>
              <p className="text-caption text-fg-medium">
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
                <label className="text-caption text-fg-medium">
                  Service Account Key (.json)
                </label>
                <Button
                  variant="secondary"
                  size="md"
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  icon={<Upload size={12} />}
                >
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
                <p className="text-caption text-fg-success-moderate mb-2">
                  ✓ Service account key already configured — load a new file or
                  paste below to replace it
                </p>
              )}
              {!jsonAlreadySet && !formFields.serviceAccountJson && (
                <p className="text-caption text-fg-moderate mb-2">
                  Paste the contents of your service-account JSON key file or
                  use "Load from file"
                </p>
              )}
              <textarea
                value={formFields.serviceAccountJson}
                onChange={(e) => setFormFields((prev) => ({ ...prev, serviceAccountJson: e.target.value }))}
                rows={6}
                className="input font-mono text-caption resize-none"
                placeholder='{"type":"service_account","project_id":"..."}'
              />
            </div>

            {error && <Banner variant="danger">{error}</Banner>}

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
