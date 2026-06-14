import { Spinner } from "../primitives/Spinner";
import { IconButton } from "../primitives/IconButton";
import { SelectableCard } from "../primitives/SelectableCard";
import { ProviderIcon } from "./ProviderIcon";
import { useEffect, useState } from "react";
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
  title = "Select Cloud Provider",
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
    <div className="flex flex-col h-full bg-gray-900">
      <div className="flex items-center gap-3 px-5 pt-5 pb-4 border-b border-gray-700/50">
        <IconButton accent="white" size="sm" onClick={onClose} className="flex-shrink-0">
          <svg
            className="w-5 h-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </IconButton>
        <div>
          <h1 className="text-base font-semibold text-primary leading-tight">
            {title}
          </h1>
          <p className="text-xs text-gray-500 mt-0.5">
            {subtitle}
          </p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-5 pt-5 pb-5">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Spinner size="w-6 h-6" color="border-blue-400" />
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {filteredProviders.map((provider) => (
              <SelectableCard
                key={provider.name}
                onClick={() => onSelectProvider(provider.name)}
                className="flex items-center gap-4 p-4 bg-gray-800/60 border border-gray-500/15 rounded-xl hover:border-blue-500/40 hover:bg-gray-800 group"
              >
                <div className="w-11 h-11 rounded-xl flex items-center justify-center p-2 flex-shrink-0">
                  <ProviderIcon provider={provider.name} className="w-full h-full" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-semibold text-primary group-hover:text-blue-300 transition-colors">
                    {provider.label}
                  </p>
                  <p className="text-xs text-gray-500 mt-0.5 truncate">
                    {provider.description}
                  </p>
                </div>
                <svg
                  className="w-4 h-4 text-gray-600 group-hover:text-blue-400 transition-colors flex-shrink-0"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                </svg>
              </SelectableCard>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
