import { useState } from "react";
import { FileText } from "lucide-react";
import { CloudProviderName, VerifiableCredentialsRequest } from "../../types";
import { PROVIDER_POLICIES, PROVIDER_SETUP_INSTRUCTIONS } from "../../constants/providerSetup";
import { CollapsibleSection } from "../primitives/CollapsibleSection";
import { AwsCredentialsForm } from "./AwsCredentialsForm";
import { AzureCredentialsForm } from "./AzureCredentialsForm";
import { GcpCredentialsForm } from "./GcpCredentialsForm";
import { OracleCredentialsForm } from "./OracleCredentialsForm";
import { PolicyBox } from "./PolicyBox";
import { SetupInstructions } from "./SetupInstructions";

interface CredentialsStepProps {
  provider: CloudProviderName;
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onCredentialsSubmit: (request: VerifiableCredentialsRequest) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

export function CredentialsStep({
  provider,
  onCredentialsSaved,
  onCredentialsSubmit,
  isSubmitting,
  onCancel,
}: CredentialsStepProps) {
  const instructions = PROVIDER_SETUP_INSTRUCTIONS[provider] ?? null;
  const policy = PROVIDER_POLICIES[provider] ?? null;
  const [isPolicyOpen, setIsPolicyOpen] = useState(false);

  return (
    <div className="mx-auto flex max-w-160 flex-col gap-3">
      {instructions && <SetupInstructions instructions={instructions} />}

      {policy && (
        <CollapsibleSection
          title="Permissions policy"
          subtitle={policy.filename}
          icon={<FileText size={16} />}
          isOpen={isPolicyOpen}
          onToggle={() => setIsPolicyOpen((previous) => !previous)}
        >
          <PolicyBox policy={policy} />
        </CollapsibleSection>
      )}

      <section className="flex flex-col gap-4 rounded-xl border border-bd-moderate bg-bg-bolder p-4">
        <h2 className="text-body-sm text-fg-lighter">Credentials</h2>
        {provider === CloudProviderName.Aws && (
          <AwsCredentialsForm
            onSubmit={(credentials) =>
              onCredentialsSubmit({ provider: CloudProviderName.Aws, credentials })
            }
            isSubmitting={isSubmitting}
            onCancel={onCancel}
          />
        )}
        {provider === CloudProviderName.Oracle && (
          <OracleCredentialsForm
            onSaved={() => onCredentialsSaved(CloudProviderName.Oracle)}
            onCancel={onCancel}
          />
        )}
        {provider === CloudProviderName.Gcp && (
          <GcpCredentialsForm
            onSubmit={(credentials) =>
              onCredentialsSubmit({ provider: CloudProviderName.Gcp, credentials })
            }
            isSubmitting={isSubmitting}
            onCancel={onCancel}
          />
        )}
        {provider === CloudProviderName.Azure && (
          <AzureCredentialsForm
            onSubmit={(credentials) =>
              onCredentialsSubmit({ provider: CloudProviderName.Azure, credentials })
            }
            isSubmitting={isSubmitting}
            onCancel={onCancel}
          />
        )}
      </section>
    </div>
  );
}
