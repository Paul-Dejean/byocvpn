import { useEffect, useRef, useState } from "react";
import { useCredentials } from "../../hooks";
import { CloudProviderName } from "../../types";
import { Upload } from "lucide-react";
import { Button } from "../primitives/Button";
import { AccountIconTile, AccountRow } from "./AccountRow";
import { Banner } from "../primitives/Banner";
import { FormField } from "../primitives/FormField";

interface OracleAccountCardProps {
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: CloudProviderName) => void;
  isProvisioned: boolean;
}


export function OracleAccountCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: OracleAccountCardProps) {
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

  return (
    <div>
      {!isEditing ? (
        <AccountRow
          provider={CloudProviderName.Oracle}
          title="Oracle Cloud Account"
          hasCredentials={hasCredentials}
          isProvisioned={isProvisioned}
          isConfirmingDelete={isConfirmingDelete}
          onEdit={handleEditOpen}
          onProvision={() => onProvisionRequested(CloudProviderName.Oracle)}
          onRequestDelete={() => setIsConfirmingDelete(true)}
          onCancelDelete={() => setIsConfirmingDelete(false)}
          onConfirmDelete={handleDeleteCredentials}
        />
      ) : (
        <div className="p-4 flex flex-col gap-5">
          <div className="flex items-center gap-3">
            <AccountIconTile provider={CloudProviderName.Oracle} />
            <div>
              <h3 className="text-body-sm text-fg-lighter">
                {hasCredentials
                  ? "Edit Oracle Cloud Account"
                  : "Add Oracle Cloud Account"}
              </h3>
              <p className="text-caption text-fg-medium">
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
                <label className="text-caption text-fg-medium">
                  Private Key (.pem)
                </label>
                <Button
                  variant="secondary"
                  size="md"
                  type="button"
                  onClick={() => pemFileInputRef.current?.click()}
                  icon={<Upload size={12} />}
                >
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
                <p className="text-caption text-fg-success-moderate mb-2">
                  ✓ Private key already configured — load a new file or paste
                  below to replace it
                </p>
              )}
              {!pemAlreadySet && !formFields.privateKeyPem && (
                <p className="text-caption text-fg-moderate mb-2">
                  Paste the contents of your .pem file or use "Load from file"
                </p>
              )}
              <textarea
                value={formFields.privateKeyPem}
                onChange={(e) => setFormFields((prev) => ({ ...prev, privateKeyPem: e.target.value }))}
                rows={6}
                className="input font-mono text-caption resize-none"
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
