import { useState } from "react";
import { CloudProviderName } from "../../types";
import { useCredentials } from "../../hooks/useCredentials";
import { FormField } from "../primitives/FormField";
import { CredentialsFormShell } from "./CredentialsFormShell";
import { KeyFileField } from "./KeyFileField";

interface OracleCredentialsFormProps {
  onSaved: () => void;
  onCancel: () => void;
}

export function OracleCredentialsForm({ onSaved, onCancel }: OracleCredentialsFormProps) {
  const [tenancyOcid, setTenancyOcid] = useState("");
  const [userOcid, setUserOcid] = useState("");
  const [fingerprint, setFingerprint] = useState("");
  const [homeRegion, setHomeRegion] = useState("");
  const [privateKeyPem, setPrivateKeyPem] = useState("");
  const { isSaving, error, saveCredentials } = useCredentials();

  const handleSubmit = async () => {
    const success = await saveCredentials({
      provider: CloudProviderName.Oracle,
      tenancyOcid: tenancyOcid.trim(),
      userOcid: userOcid.trim(),
      fingerprint: fingerprint.trim(),
      privateKeyPem: privateKeyPem.trim(),
      region: homeRegion.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid =
    tenancyOcid.trim() &&
    userOcid.trim() &&
    fingerprint.trim() &&
    homeRegion.trim() &&
    privateKeyPem.trim();

  return (
    <CredentialsFormShell
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Tenancy OCID"
        hint="e.g. ocid1.tenancy.oc1..aaaaaa…"
        type="text"
        mono
        value={tenancyOcid}
        onChange={setTenancyOcid}
      />
      <FormField
        label="User OCID"
        hint="e.g. ocid1.user.oc1..aaaaaa…"
        type="text"
        mono
        value={userOcid}
        onChange={setUserOcid}
      />
      <FormField
        label="Key Fingerprint"
        hint="e.g. xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx"
        type="text"
        mono
        value={fingerprint}
        onChange={setFingerprint}
      />
      <FormField
        label="Home Region"
        hint="e.g. us-ashburn-1"
        type="text"
        mono
        value={homeRegion}
        onChange={setHomeRegion}
      />
      <KeyFileField
        label="Private Key (.pem)"
        accept=".pem"
        value={privateKeyPem}
        onChange={setPrivateKeyPem}
        rows={5}
        placeholder="-----BEGIN RSA PRIVATE KEY-----"
      />
    </CredentialsFormShell>
  );
}
