import { useEffect, useState } from "react";
import { CloudProviderName, Region } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { getCountryName } from "../../lib/countryName";
import {
  DeploymentStatus,
  useRegionDeployment,
} from "../../hooks/useRegionDeployment";
import { Modal } from "../primitives/Modal";
import { Button } from "../primitives/Button";
import { RegionList } from "../regions/RegionList";
import { DeploymentProgress } from "./DeploymentProgress";
import { ProviderDropdown } from "./ProviderDropdown";

interface DeployServerModalProps {
  isOpen: boolean;
  configuredProviders: CloudProviderName[];
  onClose: () => void;
}

export function DeployServerModal({
  isOpen,
  configuredProviders,
  onClose,
}: DeployServerModalProps) {
  const [provider, setProvider] = useState<CloudProviderName>(
    configuredProviders[0] ?? CloudProviderName.Aws,
  );

  useEffect(() => {
    if (!configuredProviders.includes(provider) && configuredProviders.length > 0) {
      setProvider(configuredProviders[0]);
    }
  }, [configuredProviders, provider]);

  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      {isOpen && (
        <DeployServerModalContent
          key={provider}
          provider={provider}
          configuredProviders={configuredProviders}
          onSelectProvider={setProvider}
          onClose={onClose}
        />
      )}
    </Modal>
  );
}

interface DeployServerModalContentProps {
  provider: CloudProviderName;
  configuredProviders: CloudProviderName[];
  onSelectProvider: (provider: CloudProviderName) => void;
  onClose: () => void;
}

function DeployServerModalContent({
  provider,
  configuredProviders,
  onSelectProvider,
  onClose,
}: DeployServerModalContentProps) {
  const deployment = useRegionDeployment(provider);

  useEffect(() => {
    if (deployment.status === DeploymentStatus.COMPLETE) {
      onClose();
    }
  }, [deployment.status, onClose]);

  if (deployment.status !== DeploymentStatus.IDLE && deployment.region) {
    return (
      <div className="p-6 flex flex-col gap-10 min-h-[560px]">
        <div>
          <Button
            variant="secondary"
            size="none"
            onClick={onClose}
            className="px-3 py-1.5 text-sm"
          >
            Cancel
          </Button>
        </div>
        <DeploymentProgress
          title={`Setting up your ${describeRegion(provider, deployment.region)} server`}
          steps={deployment.steps}
          error={deployment.error}
        />
      </div>
    );
  }

  return (
    <div className="p-6 flex flex-col gap-5 min-h-0 max-h-[640px]">
      <header className="flex flex-col gap-1">
        <h2 className="text-base font-medium text-primary">Select servers</h2>
        <p className="text-xs text-gray-300">
          Choose a region to deploy your VPN server.
        </p>
      </header>

      <RegionList
        provider={provider}
        onDeployRegion={deployment.deploy}
        headerAccessory={
          <ProviderDropdown
            providers={configuredProviders}
            selectedProvider={provider}
            onSelectProvider={onSelectProvider}
          />
        }
      />
    </div>
  );
}

function describeRegion(provider: CloudProviderName, region: Region): string {
  const regionInfo = getRegionInfo(provider, region.name);
  return getCountryName(regionInfo.countryCode) || region.country;
}
