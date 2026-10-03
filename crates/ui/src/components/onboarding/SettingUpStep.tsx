import { useEffect, useRef } from "react";
import { CloudProviderName, Instance, Region } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { getCountryName } from "../../lib/countryName";
import {
  DeploymentStatus,
  useRegionDeployment,
} from "../../hooks/useRegionDeployment";
import { Logo } from "../common/Logo";
import { Button } from "../primitives/Button";
import { DeploymentProgress } from "../deploy/DeploymentProgress";

interface SettingUpStepProps {
  provider: CloudProviderName;
  region: Region;
  onComplete: (instance: Instance) => void;
  onBack: () => void;
}

export function SettingUpStep({
  provider,
  region,
  onComplete,
  onBack,
}: SettingUpStepProps) {
  const deployment = useRegionDeployment(provider);
  const hasStartedDeployment = useRef(false);

  useEffect(() => {
    if (!hasStartedDeployment.current) {
      hasStartedDeployment.current = true;
      deployment.deploy(region);
    }
  }, []);

  useEffect(() => {
    if (deployment.status === DeploymentStatus.COMPLETE && deployment.instance) {
      onComplete(deployment.instance);
    }
  }, [deployment.status, deployment.instance, onComplete]);

  const regionInfo = getRegionInfo(provider, region.name);
  const countryName = getCountryName(regionInfo.countryCode) || region.country;

  function handleBack() {
    deployment.reset();
    onBack();
  }

  return (
    <div className="h-full flex flex-col items-center justify-center gap-8">
      <div className="w-12 h-12 rounded-xl bg-bg-medium border border-bd-moderate flex items-center justify-center">
        <Logo className="w-6 h-6" />
      </div>

      <DeploymentProgress
        title={`Setting up your ${countryName} server`}
        steps={deployment.steps}
        error={deployment.error}
      />

      {deployment.status === DeploymentStatus.FAILED && (
        <Button
          variant="secondary"
          size="lg"
          onClick={handleBack}
        >
          Back
        </Button>
      )}
    </div>
  );
}
