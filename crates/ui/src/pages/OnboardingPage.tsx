import { useState } from "react";
import { CloudProviderName } from "../types";
import { WelcomeStep } from "../components/onboarding/WelcomeStep";
import { ConnectAccountStep } from "../components/onboarding/ConnectAccountStep";

enum OnboardingStep {
  WELCOME = "WELCOME",
  CONNECT_ACCOUNT = "CONNECT_ACCOUNT",
}

interface OnboardingPageProps {
  onSkip: () => void;
  onProviderSelected: (provider: CloudProviderName) => void;
}

export function OnboardingPage({
  onSkip,
  onProviderSelected,
}: OnboardingPageProps) {
  const [step, setStep] = useState<OnboardingStep>(OnboardingStep.WELCOME);

  if (step === OnboardingStep.CONNECT_ACCOUNT) {
    return (
      <ConnectAccountStep
        onBack={() => setStep(OnboardingStep.WELCOME)}
        onSkip={onSkip}
        onContinue={onProviderSelected}
      />
    );
  }

  return (
    <WelcomeStep onGetStarted={() => setStep(OnboardingStep.CONNECT_ACCOUNT)} />
  );
}
