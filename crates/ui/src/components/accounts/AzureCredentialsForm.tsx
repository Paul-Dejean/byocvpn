import { useState } from "react";
import { AzureCredentials } from "../../types";
import { FormField } from "../primitives/FormField";
import { CredentialsFormShell } from "./CredentialsFormShell";

interface AzureCredentialsFormProps {
  onSubmit: (credentials: AzureCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

export function AzureCredentialsForm({
  onSubmit,
  isSubmitting,
  onCancel,
}: AzureCredentialsFormProps) {
  const [subscriptionId, setSubscriptionId] = useState("");
  const [tenantId, setTenantId] = useState("");
  const [applicationId, setApplicationId] = useState("");
  const [secretValue, setSecretValue] = useState("");

  const handleSubmit = () => {
    onSubmit({
      subscriptionId: subscriptionId.trim(),
      tenantId: tenantId.trim(),
      applicationId: applicationId.trim(),
      secretValue: secretValue.trim(),
    });
  };

  const isFormValid =
    subscriptionId.trim() && tenantId.trim() && applicationId.trim() && secretValue.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Subscription ID"
        hint="e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
        type="text"
        mono
        value={subscriptionId}
        onChange={setSubscriptionId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Tenant ID"
        hint="Your Azure Active Directory tenant ID"
        type="text"
        mono
        value={tenantId}
        onChange={setTenantId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Application ID"
        hint="Application ID of the service principal"
        type="text"
        mono
        value={applicationId}
        onChange={setApplicationId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Secret Value"
        hint="Secret value from your app registration"
        type="password"
        mono
        value={secretValue}
        onChange={setSecretValue}
      />
    </CredentialsFormShell>
  );
}
