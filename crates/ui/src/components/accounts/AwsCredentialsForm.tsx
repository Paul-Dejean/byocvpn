import { useState } from "react";
import { AwsCredentials } from "../../types";
import { FormField } from "../primitives/FormField";
import { CredentialsFormShell } from "./CredentialsFormShell";

interface AwsCredentialsFormProps {
  onSubmit: (credentials: AwsCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

export function AwsCredentialsForm({ onSubmit, isSubmitting, onCancel }: AwsCredentialsFormProps) {
  const [accessKeyId, setAccessKeyId] = useState("");
  const [secretAccessKey, setSecretAccessKey] = useState("");

  const handleSubmit = () => {
    onSubmit({
      accessKeyId: accessKeyId.trim(),
      secretAccessKey: secretAccessKey.trim(),
    });
  };

  const isFormValid = accessKeyId.trim() && secretAccessKey.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Access Key ID"
        hint="e.g. AKIAIOSFODNN7EXAMPLE"
        type="text"
        mono
        value={accessKeyId}
        onChange={setAccessKeyId}
      />
      <FormField
        label="Secret Access Key"
        type="password"
        value={secretAccessKey}
        onChange={setSecretAccessKey}
      />
    </CredentialsFormShell>
  );
}
