import { useState } from "react";
import { CloudProviderName, Instance, Region } from "../types";
import { OnboardingStep } from "../types/onboarding";
import { WelcomeStep } from "../components/onboarding/WelcomeStep";
import { ConnectAccountStep } from "../components/onboarding/ConnectAccountStep";
import { DeployServerStep } from "../components/onboarding/DeployServerStep";
import { SettingUpStep } from "../components/onboarding/SettingUpStep";
import { ReadyStep } from "../components/onboarding/ReadyStep";

interface OnboardingPageProps {
  initialStep?: OnboardingStep;
  provider: CloudProviderName | null;
  onSkip: () => void;
  onProviderSelected: (provider: CloudProviderName) => void;
  onFinished: () => void;
}

export function OnboardingPage({
  initialStep = OnboardingStep.WELCOME,
  provider,
  onSkip,
  onProviderSelected,
  onFinished,
}: OnboardingPageProps) {
  const [step, setStep] = useState<OnboardingStep>(initialStep);
  const [region, setRegion] = useState<Region | null>(null);
  const [instance, setInstance] = useState<Instance | null>(null);

  if (step === OnboardingStep.CONNECT_ACCOUNT) {
    return (
      <ConnectAccountStep
        onBack={() => setStep(OnboardingStep.WELCOME)}
        onSkip={onSkip}
        onContinue={onProviderSelected}
      />
    );
  }

  if (step === OnboardingStep.DEPLOY_SERVER && provider) {
    return (
      <DeployServerStep
        provider={provider}
        onBack={() => setStep(OnboardingStep.CONNECT_ACCOUNT)}
        onSkip={onSkip}
        onNext={(selectedRegion) => {
          setRegion(selectedRegion);
          setStep(OnboardingStep.SETTING_UP);
        }}
      />
    );
  }

  if (step === OnboardingStep.SETTING_UP && provider && region) {
    return (
      <SettingUpStep
        provider={provider}
        region={region}
        onComplete={(launchedInstance) => {
          setInstance(launchedInstance);
          setStep(OnboardingStep.READY);
        }}
        onBack={() => setStep(OnboardingStep.DEPLOY_SERVER)}
      />
    );
  }

  if (step === OnboardingStep.READY && instance) {
    return <ReadyStep instance={instance} onConnected={onFinished} />;
  }

  return (
    <WelcomeStep onGetStarted={() => setStep(OnboardingStep.CONNECT_ACCOUNT)} />
  );
}
