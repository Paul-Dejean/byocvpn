import { useEffect, useState } from "react";
import { CloudProviderName, Region } from "../../types";
import { useDeployments } from "../../contexts/DeploymentsContext";
import { Modal } from "../primitives/Modal";
import { RegionList } from "../regions/RegionList";
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
  const { startDeployment } = useDeployments();
  const [provider, setProvider] = useState<CloudProviderName>(
    configuredProviders[0] ?? CloudProviderName.Aws,
  );

  useEffect(() => {
    if (!configuredProviders.includes(provider) && configuredProviders.length > 0) {
      setProvider(configuredProviders[0]);
    }
  }, [configuredProviders, provider]);

  function handleDeploy(region: Region) {
    startDeployment(provider, region);
    onClose();
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      <div className="p-6 flex flex-col gap-5 min-h-0 max-h-[640px]">
        <header className="flex flex-col gap-1">
          <h2 className="text-base font-medium text-primary">Select servers</h2>
          <p className="text-xs text-gray-300">
            Choose a region to deploy your VPN server.
          </p>
        </header>

        <RegionList
          provider={provider}
          onDeployRegion={handleDeploy}
          headerAccessory={
            <ProviderDropdown
              providers={configuredProviders}
              selectedProvider={provider}
              onSelectProvider={setProvider}
            />
          }
        />
      </div>
    </Modal>
  );
}
