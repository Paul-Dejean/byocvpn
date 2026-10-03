import { useEffect, useState } from "react";
import { ArrowLeft, ChevronRight } from "lucide-react";
import { Spinner } from "../primitives/Spinner";
import { IconButton } from "../primitives/IconButton";
import { ProviderIcon } from "./ProviderIcon";
import { useCredentials } from "../../hooks/useCredentials";
import { CloudProviderName } from "../../types";

interface ProviderSelectorProps {
  onSelectProvider: (provider: CloudProviderName) => void;
  onClose: () => void;
  filter?: "configured" | "unconfigured";
  title?: string;
  subtitle?: string;
}

interface ProviderOption {
  name: CloudProviderName;
  label: string;
  description: string;
}

const providers: ProviderOption[] = [
  {
    name: CloudProviderName.Aws,
    label: "Amazon Web Services",
    description: "EC2 — 15+ regions worldwide",
  },
  {
    name: CloudProviderName.Oracle,
    label: "Oracle Cloud",
    description: "OCI Compute — 40+ regions worldwide",
  },
  {
    name: CloudProviderName.Gcp,
    label: "Google Cloud",
    description: "Compute Engine — 40+ regions worldwide",
  },
  {
    name: CloudProviderName.Azure,
    label: "Microsoft Azure",
    description: "Azure VMs — 60+ regions worldwide",
  },
];

export function ProviderSelector({
  onSelectProvider,
  onClose,
  filter = "configured",
  title = "Select cloud provider",
  subtitle = "Choose which provider to deploy your VPN server on",
}: ProviderSelectorProps) {
  const [filteredProviders, setFilteredProviders] = useState<ProviderOption[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const { loadCredentials } = useCredentials();

  useEffect(() => {
    const loadFilteredProviders = async () => {
      const result: ProviderOption[] = [];
      for (const provider of providers) {
        const existing = await loadCredentials(provider.name);
        const shouldInclude = filter === "unconfigured" ? existing === null : existing !== null;
        if (shouldInclude) {
          result.push(provider);
        }
      }
      setFilteredProviders(result);
      setIsLoading(false);
    };
    loadFilteredProviders();
  }, []);

  return (
    <div className="flex flex-col h-full gap-4">
      <header className="flex items-center gap-3">
        <IconButton accent="white" size="sm" onClick={onClose} aria-label="Back">
          <ArrowLeft size={18} />
        </IconButton>
        <div className="flex flex-col gap-0.5">
          <h1 className="text-base font-medium text-primary">{title}</h1>
          <p className="text-xs text-gray-300">{subtitle}</p>
        </div>
      </header>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Spinner size="w-6 h-6" color="border-gray-400" />
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3 max-w-2xl">
            {filteredProviders.map((provider) => (
              <button
                key={provider.name}
                type="button"
                onClick={() => onSelectProvider(provider.name)}
                className="group flex items-center gap-3 p-4 rounded-xl bg-gray-750 border border-gray-500/50 hover:bg-gray-700 hover:border-gray-500 text-left transition-colors"
              >
                <div className="w-10 h-10 rounded-lg bg-gray-700 border border-gray-500/60 flex items-center justify-center p-2 flex-shrink-0">
                  <ProviderIcon provider={provider.name} className="w-full h-full" />
                </div>
                <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                  <span className="text-sm text-primary">{provider.label}</span>
                  <span className="text-xs text-gray-300 truncate">{provider.description}</span>
                </div>
                <ChevronRight
                  size={16}
                  className="text-gray-400 group-hover:text-primary transition-colors flex-shrink-0"
                />
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
