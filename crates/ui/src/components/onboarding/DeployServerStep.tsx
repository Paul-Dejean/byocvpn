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

export function DeployServerStep({
  provider,
  onBack,
  onSkip,
  onNext,
}: DeployServerStepProps) {
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);

  function handleNext() {
    if (selectedRegion) {
      onNext(selectedRegion);
    }
  }

  return (
    <div className="h-full flex flex-col items-center py-10 gap-8">
      <OnboardingHeading
        title="Deploy your first server to stay private"
        subtitle="Your credentials stay on your device. We never store them."
      />

      <div className="w-[400px] flex-1 min-h-0 flex flex-col">
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
          className="text-caption text-fg-lighter hover:text-fg-medium transition-colors"
        >
          I'll do this later
        </button>

        <div className="flex items-center gap-4">
          <button
            type="button"
            onClick={onBack}
            className="px-2 py-2 text-body-sm text-fg-lighter hover:text-fg-medium transition-colors"
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
