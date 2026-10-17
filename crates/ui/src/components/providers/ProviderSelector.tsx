import { useEffect, useState } from "react";
import { CloudProviderName } from "../../types";
import { useCredentials } from "../../hooks/useCredentials";
import { OnboardingHeading } from "../onboarding/OnboardingHeading";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { ProviderTile } from "./ProviderTile";
import { PROVIDER_TILE_LABELS } from "./ProviderTile/labels";

interface ProviderSelectorProps {
  onSelectProvider: (provider: CloudProviderName) => void;
  onClose: () => void;
  filter?: "configured" | "unconfigured";
  title?: string;
  subtitle?: string;
}

const ALL_PROVIDERS: CloudProviderName[] = [
  CloudProviderName.Aws,
  CloudProviderName.Oracle,
  CloudProviderName.Gcp,
  CloudProviderName.Azure,
];

export function ProviderSelector({
  onSelectProvider,
  onClose,
  filter = "configured",
  title = "Select a cloud account",
  subtitle = "Your credentials stay on your device. We never store them.",
}: ProviderSelectorProps) {
  const [availableProviders, setAvailableProviders] = useState<CloudProviderName[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { loadCredentials } = useCredentials();

  useEffect(() => {
    async function loadAvailableProviders() {
      const providerCredentials = await Promise.all(
        ALL_PROVIDERS.map(async (provider) => ({
          provider,
          existing: await loadCredentials(provider),
        })),
      );
      const result = providerCredentials
        .filter(({ existing }) =>
          filter === "unconfigured" ? existing === null : existing !== null,
        )
        .map(({ provider }) => provider);
      setAvailableProviders(result);
      setIsLoading(false);
    }
    void loadAvailableProviders();
  }, [filter, loadCredentials]);

  function handleContinue() {
    if (selectedProvider) {
      onSelectProvider(selectedProvider);
    }
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-8">
      <OnboardingHeading title={title} subtitle={subtitle} />

      {isLoading ? (
        <div className="flex h-42 items-center justify-center">
          <Spinner size="w-6 h-6" color="border-bd-strong" />
        </div>
      ) : availableProviders.length === 0 ? (
        <p className="text-body-sm text-fg-medium">
          Every supported cloud provider is already connected.
        </p>
      ) : (
        <div className="grid grid-cols-2 gap-3">
          {availableProviders.map((provider) => (
            <ProviderTile
              key={provider}
              provider={provider}
              label={PROVIDER_TILE_LABELS[provider]}
              isSelected={selectedProvider === provider}
              onSelect={() => setSelectedProvider(provider)}
            />
          ))}
        </div>
      )}

      <div className="flex items-center gap-4">
        <button
          type="button"
          onClick={onClose}
          className="p-2 text-body-sm text-fg-lighter transition-colors hover:text-fg-medium"
        >
          Back
        </button>
        <Button
          variant="primary"
          size="lg"
          disabled={selectedProvider === null}
          onClick={handleContinue}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
