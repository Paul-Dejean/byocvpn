import { useState } from "react";
import { CloudProviderName, Region } from "../../types";
import { Button } from "../primitives/Button";
import { RegionList } from "../regions/RegionList";
import { OnboardingHeading } from "./OnboardingHeading";

interface DeployServerStepProps {
  provider: CloudProviderName;
  onBack: () => void;
  onSkip: () => void;
  onNext: (region: Region) => void;
}

export function DeployServerStep({ provider, onBack, onSkip, onNext }: DeployServerStepProps) {
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);

  function handleNext() {
    if (selectedRegion) {
      onNext(selectedRegion);
    }
  }

  return (
    <div className="flex h-full flex-col items-center gap-8 py-10 max-md:gap-6 max-md:py-4">
      <OnboardingHeading
        title="Deploy your first server to stay private"
        subtitle="Your credentials stay on your device. We never store them."
      />

      <div className="flex min-h-0 w-full max-w-100 flex-1 flex-col">
        <RegionList
          provider={provider}
          selectedRegion={selectedRegion}
          onSelectRegion={setSelectedRegion}
        />
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
            disabled={selectedRegion === null}
            onClick={handleNext}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
