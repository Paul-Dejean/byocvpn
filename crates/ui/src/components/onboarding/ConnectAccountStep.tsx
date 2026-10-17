import { useState } from "react";
import { CloudProviderName } from "../../types";
import { Button } from "../primitives/Button";
import { ProviderTile } from "../providers/ProviderTile";
import { PROVIDER_TILE_LABELS } from "../providers/ProviderTile/labels";
import { OnboardingHeading } from "./OnboardingHeading";

interface ConnectAccountStepProps {
  onBack: () => void;
  onSkip: () => void;
  onContinue: (provider: CloudProviderName) => void;
}

const PROVIDER_TILES: CloudProviderName[] = [
  CloudProviderName.Aws,
  CloudProviderName.Oracle,
  CloudProviderName.Gcp,
  CloudProviderName.Azure,
];

export function ConnectAccountStep({ onBack, onSkip, onContinue }: ConnectAccountStepProps) {
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName | null>(null);

  function handleContinue() {
    if (selectedProvider) {
      onContinue(selectedProvider);
    }
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-8">
      <OnboardingHeading
        title="Connect your first cloud account"
        subtitle="Your credentials stay on your device. We never store them."
      />

      <div className="grid grid-cols-2 gap-3">
        {PROVIDER_TILES.map((provider) => (
          <ProviderTile
            key={provider}
            provider={provider}
            label={PROVIDER_TILE_LABELS[provider]}
            isSelected={selectedProvider === provider}
            onSelect={() => setSelectedProvider(provider)}
          />
        ))}
      </div>

      <div className="flex flex-col items-center gap-10">
        <button
          type="button"
          onClick={onSkip}
          className="text-caption text-fg-lighter transition-colors hover:text-fg-medium"
        >
          I'll do this later
        </button>

        <div className="flex items-center gap-4">
          <button
            type="button"
            onClick={onBack}
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
    </div>
  );
}
