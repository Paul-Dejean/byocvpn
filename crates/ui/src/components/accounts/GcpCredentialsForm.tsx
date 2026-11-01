import { useState } from "react";
import { GcpCredentials } from "../../types";
import { readProjectIdFromServiceAccountJson } from "../../lib/serviceAccountJson";
import { FormField } from "../primitives/FormField";
import { CredentialsFormShell } from "./CredentialsFormShell";
import { KeyFileField } from "./KeyFileField";

interface GcpCredentialsFormProps {
  onSubmit: (credentials: GcpCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

export function GcpCredentialsForm({ onSubmit, isSubmitting, onCancel }: GcpCredentialsFormProps) {
  const [projectId, setProjectId] = useState("");
  const [serviceAccountJson, setServiceAccountJson] = useState("");

  const handleFileLoaded = (content: string) => {
    const parsedProjectId = readProjectIdFromServiceAccountJson(content);
    if (parsedProjectId && !projectId) setProjectId(parsedProjectId);
  };

  const handleSubmit = () => {
    onSubmit({
      projectId: projectId.trim(),
      serviceAccountJson: serviceAccountJson.trim(),
    });
  };

  const isFormValid = projectId.trim() && serviceAccountJson.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Project ID"
        hint="e.g. my-project-123456"
        type="text"
        mono
        value={projectId}
        onChange={setProjectId}
      />
      <KeyFileField
        label="Service Account Key (.json)"
        accept=".json,application/json"
        value={serviceAccountJson}
        onChange={setServiceAccountJson}
        onFileLoaded={handleFileLoaded}
        placeholder='{"type":"service_account","project_id":"..."}'
      />
    </CredentialsFormShell>
  );
}
