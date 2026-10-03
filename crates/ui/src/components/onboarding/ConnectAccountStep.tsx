import { useState } from "react";
import { CloudProviderName } from "../../types";
import { Button } from "../primitives/Button";
import { ProviderIcon } from "../providers/ProviderIcon";
import { OnboardingHeading } from "./OnboardingHeading";

interface ConnectAccountStepProps {
  onBack: () => void;
  onSkip: () => void;
  onContinue: (provider: CloudProviderName) => void;
}

interface ProviderTile {
  provider: CloudProviderName;
  label: string;
}

const PROVIDER_TILES: ProviderTile[] = [
  { provider: CloudProviderName.Aws, label: "AWS Account" },
  { provider: CloudProviderName.Oracle, label: "Oracle Cloud" },
  { provider: CloudProviderName.Gcp, label: "Google Cloud" },
  { provider: CloudProviderName.Azure, label: "Microsoft Azure" },
];

export function ConnectAccountStep({
  onBack,
  onSkip,
  onContinue,
}: ConnectAccountStepProps) {
  const [selectedProvider, setSelectedProvider] =
    useState<CloudProviderName | null>(null);

  function handleContinue() {
    if (selectedProvider) {
      onContinue(selectedProvider);
    }
  }

  return (
    <div className="h-full flex flex-col items-center justify-center gap-8">
      <OnboardingHeading
        title="Connect your first cloud account"
        subtitle="Your credentials stay on your device. We never store them."
      />

      <div className="grid grid-cols-2 gap-3">
        {PROVIDER_TILES.map((tile) => (
          <ProviderTileButton
            key={tile.provider}
            tile={tile}
            isSelected={selectedProvider === tile.provider}
            onSelect={() => setSelectedProvider(tile.provider)}
          />
        ))}
      </div>

      <div className="flex flex-col items-center gap-6">
        <button
          type="button"
          onClick={onSkip}
          className="text-xs text-primary hover:text-gray-300 transition-colors"
        >
          I'll do this later
        </button>

        <div className="flex items-center gap-4">
          <button
            type="button"
            onClick={onBack}
            className="px-2 py-2 text-sm text-primary hover:text-gray-300 transition-colors"
          >
            Back
          </button>
          <Button
            variant="primary"
            size="none"
            disabledStyle="dim"
            disabled={selectedProvider === null}
            onClick={handleContinue}
            className="px-4 py-2 text-sm"
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}

interface ProviderTileButtonProps {
  tile: ProviderTile;
  isSelected: boolean;
  onSelect: () => void;
}

function ProviderTileButton({
  tile,
  isSelected,
  onSelect,
}: ProviderTileButtonProps) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={`w-[124px] h-[78px] rounded-lg border p-3 flex flex-col justify-between items-start text-left transition-colors ${
        isSelected
          ? "bg-gray-700 border-blue-500"
          : "bg-gray-750 border-gray-500/60 hover:bg-gray-700 hover:border-gray-500"
      }`}
    >
      <ProviderIcon provider={tile.provider} className="w-6 h-6" />
      <span className="text-xs text-primary">{tile.label}</span>
    </button>
  );
}
