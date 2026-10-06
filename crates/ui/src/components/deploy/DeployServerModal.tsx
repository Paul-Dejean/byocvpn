import { X } from "lucide-react";
import { CloudProviderName, Region } from "../../types";
import { useDeployments } from "../../contexts/DeploymentsContext";
import { useLastDeployProvider } from "../../hooks/useLastDeployProvider";
import { IconButton } from "../primitives/IconButton";
import { Modal } from "../primitives/Modal";
import { Spinner } from "../primitives/Spinner";
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
  const { provider, isLoading, selectProvider } =
    useLastDeployProvider(configuredProviders);

  function handleDeploy(region: Region) {
    if (!provider) {
      return;
    }
    startDeployment(provider, region);
    onClose();
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      <div className="p-6 max-md:px-4 max-md:pt-1 flex flex-col gap-4 min-h-0 max-h-[640px] max-md:max-h-none max-md:flex-1">
        <header className="flex items-start justify-between gap-4">
          <div className="flex flex-col gap-1">
            <h2 className="text-body-sm font-medium text-fg-lighter">Select servers</h2>
            <p className="text-caption text-fg-medium">
              Choose a region to deploy your VPN server.
            </p>
          </div>
          <IconButton size="sm" onClick={onClose} aria-label="Close" className="md:hidden">
            <X size={16} />
          </IconButton>
        </header>

        {isLoading || provider === null ? (
          <div className="flex justify-center py-10">
            <Spinner size="w-6 h-6" color="border-bd-strong" />
          </div>
        ) : (
          <RegionList
            key={provider}
            provider={provider}
            onDeployRegion={handleDeploy}
            headerAccessory={
              <ProviderDropdown
                providers={configuredProviders}
                selectedProvider={provider}
                onSelectProvider={selectProvider}
              />
            }
          />
        )}
      </div>
    </Modal>
  );
}
