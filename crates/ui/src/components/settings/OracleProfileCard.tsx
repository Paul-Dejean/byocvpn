import { useEffect, useRef, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Badge } from "../primitives/Badge";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { Alert } from "../primitives/Alert";
import { FormField } from "../primitives/FormField";

interface OracleProfileCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}

function OciIcon() {
  return (
    <div className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0 p-2.5">
      <img src="/cloud-providers/oracle-icon.svg" alt="Oracle" className="w-full h-full object-contain" />
    </div>
  );
}

export function OracleProfileCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: OracleProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [pemAlreadySet, setPemAlreadySet] = useState(false);
  const [formFields, setFormFields] = useState({
    tenancyOcid: "",
    userOcid: "",
    fingerprint: "",
    privateKeyPem: "",
    region: "",
  });

  const pemFileInputRef = useRef<HTMLInputElement>(null);

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
    loadCredentials(CloudProviderName.Oracle).then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, []);

  const resetForm = () => {
    setFormFields({ tenancyOcid: "", userOcid: "", fingerprint: "", privateKeyPem: "", region: "" });
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
    const success = await saveCredentials(CloudProviderName.Oracle, {
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

  const handlePemFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const content = loadEvent.target?.result;
      if (typeof content === "string") {
        setFormFields((prev) => ({ ...prev, privateKeyPem: content }));
      }
    };
    reader.readAsText(file);
    event.target.value = "";
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

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div className="py-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <OciIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-primary">Oracle Cloud Account</h3>
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
                    <IconButton accent="blue" onClick={() => onProvisionRequested(CloudProviderName.Oracle)} title="Re-provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </IconButton>
                  ) : (
                    <IconButton accent="amber" onClick={() => onProvisionRequested(CloudProviderName.Oracle)} title="Provision">
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
            <OciIcon />
            <div>
              <h3 className="font-semibold text-lg text-primary">
                {hasCredentials
                  ? "Edit Oracle Cloud Account"
                  : "Add Oracle Cloud Account"}
              </h3>
              <p className="text-sm text-gray-400">
                {hasCredentials
                  ? "Update your OCI API signing credentials"
                  : "Enter your OCI API signing credentials"}
              </p>
            </div>
          </div>

          <div className="space-y-4">
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

            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="block text-sm font-medium text-gray-300">
                  Private Key (.pem)
                </label>
                <Button
                  variant="secondary"
                  size="none"
                  type="button"
                  onClick={() => pemFileInputRef.current?.click()}
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
                  ref={pemFileInputRef}
                  type="file"
                  accept=".pem"
                  onChange={handlePemFileChange}
                  className="hidden"
                />
              </div>
              {pemAlreadySet && !formFields.privateKeyPem && (
                <p className="text-xs text-success-400 mb-2">
                  ✓ Private key already configured — load a new file or paste
                  below to replace it
                </p>
              )}
              {!pemAlreadySet && !formFields.privateKeyPem && (
                <p className="text-xs text-gray-500 mb-2">
                  Paste the contents of your .pem file or use "Load from file"
                </p>
              )}
              <textarea
                value={formFields.privateKeyPem}
                onChange={(e) => setFormFields((prev) => ({ ...prev, privateKeyPem: e.target.value }))}
                rows={6}
                className="input font-mono text-xs resize-none"
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
