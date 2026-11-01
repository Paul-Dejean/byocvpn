import { useState } from "react";
import { ArrowLeft } from "lucide-react";
import { toast } from "react-hot-toast";
import { IconButton } from "../components/primitives/IconButton";
import { JobProgressDrawer } from "../components/common/JobProgressDrawer";
import { ProviderSelector } from "../components/providers/ProviderSelector";
import { CredentialsStep } from "../components/accounts/CredentialsStep";
import { PROVIDER_METADATA } from "../constants/providers";
import { useAccounts } from "../hooks/useAccounts";
import { useCredentials } from "../hooks/useCredentials";
import { summarizeError } from "../lib/describeError";
import {
  CloudProviderName,
  VerifiableCredentialsRequest,
  buildProviderCredentials,
} from "../types";

interface AddAccountPageProps {
  initialProvider?: CloudProviderName | null;
  onNavigateBack: () => void;
  onAccountAdded: () => void;
}

type AddAccountStep = "selecting-provider" | "entering-credentials";

export function AddAccountPage({
  initialProvider = null,
  onNavigateBack,
  onAccountAdded,
}: AddAccountPageProps) {
  const [step, setStep] = useState<AddAccountStep>(
    initialProvider ? "entering-credentials" : "selecting-provider",
  );
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName | null>(
    initialProvider,
  );

  const {
    activeProvisionJob,
    isProvisionComplete,
    provisionError,
    provisionAccount,
    resetProvisionState,
  } = useAccounts({
    onFailed: (error) => toast.error(`Account setup failed: ${summarizeError(error)}`),
  });

  const { saveCredentials } = useCredentials();
  const [isSavingCredentials, setIsSavingCredentials] = useState(false);

  const handleProviderSelected = (providerName: CloudProviderName) => {
    setSelectedProvider(providerName);
    setStep("entering-credentials");
  };

  const handleBackToProviderSelection = () => {
    setSelectedProvider(null);
    setStep("selecting-provider");
  };

  const handleCredentialsSaved = (provider: CloudProviderName) => {
    void provisionAccount(provider);
  };

  const saveAndProvision = async (request: VerifiableCredentialsRequest) => {
    setIsSavingCredentials(true);
    const saved = await saveCredentials(buildProviderCredentials(request));
    setIsSavingCredentials(false);
    if (saved) {
      void provisionAccount(request.provider);
    }
  };

  const handleCloseProvisionDrawer = () => {
    const provisioningStarted = activeProvisionJob !== null;
    const hasNoSteps = (activeProvisionJob?.steps ?? []).length === 0;
    resetProvisionState();
    if (provisioningStarted && (isProvisionComplete || hasNoSteps)) {
      onAccountAdded();
    }
  };

  if (step === "selecting-provider") {
    return (
      <ProviderSelector
        filter="unconfigured"
        title="Connect a cloud account"
        subtitle="Your credentials stay on your device. We never store them."
        onSelectProvider={handleProviderSelected}
        onClose={onNavigateBack}
      />
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <header className="flex items-center gap-3">
        <IconButton
          accent="neutral"
          size="sm"
          onClick={handleBackToProviderSelection}
          aria-label="Back"
        >
          <ArrowLeft size={16} />
        </IconButton>
        <div className="flex flex-col gap-0.5">
          <h1 className="text-body-sm font-medium text-fg-lighter">
            {selectedProvider ? `Connect ${PROVIDER_METADATA[selectedProvider].label}` : ""}
          </h1>
          <p className="text-caption text-fg-medium">
            Your credentials stay on your device. We never store them.
          </p>
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        {selectedProvider && (
          <CredentialsStep
            provider={selectedProvider}
            onCredentialsSaved={handleCredentialsSaved}
            onCredentialsSubmit={saveAndProvision}
            isSubmitting={isSavingCredentials}
            onCancel={handleBackToProviderSelection}
          />
        )}
      </div>

      <JobProgressDrawer
        isOpen={activeProvisionJob !== null}
        onClose={handleCloseProvisionDrawer}
        provider={activeProvisionJob?.provider ?? selectedProvider ?? CloudProviderName.Aws}
        steps={activeProvisionJob?.steps ?? []}
        isComplete={isProvisionComplete}
        error={provisionError}
      />
    </div>
  );
}
