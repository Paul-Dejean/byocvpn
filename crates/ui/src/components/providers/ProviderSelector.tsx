import { useEffect, useState } from "react";
import { CloudProviderName } from "../../types";
import { useCredentials } from "../../hooks/useCredentials";
import { OnboardingHeading } from "../onboarding/OnboardingHeading";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { PROVIDER_TILE_LABELS, ProviderTile } from "./ProviderTile";

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
      const result: CloudProviderName[] = [];
      for (const provider of ALL_PROVIDERS) {
        const existing = await loadCredentials(provider);
        const shouldInclude = filter === "unconfigured" ? existing === null : existing !== null;
        if (shouldInclude) {
          result.push(provider);
        }
      }
      setAvailableProviders(result);
      setIsLoading(false);
    }
    loadAvailableProviders();
  }, []);

  function handleContinue() {
    if (selectedProvider) {
      onSelectProvider(selectedProvider);
    }
  }

  return (
    <div className="h-full flex flex-col items-center justify-center gap-8">
      <OnboardingHeading title={title} subtitle={subtitle} />

      {isLoading ? (
        <div className="h-[168px] flex items-center justify-center">
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
          className="px-2 py-2 text-body-sm text-fg-lighter hover:text-fg-medium transition-colors"
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
