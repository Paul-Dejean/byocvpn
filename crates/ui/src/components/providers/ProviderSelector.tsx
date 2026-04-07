import React, { useEffect, useState } from "react";
import { useCredentials } from "../../hooks/useCredentials";

interface ProviderSelectorProps {
  onSelectProvider: (provider: string) => void;
  onClose: () => void;
}

interface ProviderOption {
  id: string;
  label: string;
  description: string;
  badge: React.ReactNode;
}

const providers: ProviderOption[] = [
  {
    id: "aws",
    label: "Amazon Web Services",
    description: "Deploy on EC2 — available in 30+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-white/5 flex items-center justify-center p-2">
        <img src="/cloud-providers/aws-icon.svg" alt="AWS" className="w-full h-full object-contain" />
      </div>
    ),
  },
  {
    id: "oracle",
    label: "Oracle Cloud Infrastructure",
    description: "Deploy on OCI Compute — includes an Always Free tier",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-white/5 flex items-center justify-center p-2">
        <img src="/cloud-providers/oracle-icon.svg" alt="Oracle" className="w-full h-full object-contain" />
      </div>
    ),
  },
  {
    id: "gcp",
    label: "Google Cloud Platform",
    description:
      "Deploy on GCP Compute Engine — available in 40+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-white/5 flex items-center justify-center p-2">
        <img src="/cloud-providers/google-cloud-icon.svg" alt="GCP" className="w-full h-full object-contain" />
      </div>
    ),
  },
  {
    id: "azure",
    label: "Microsoft Azure",
    description: "Deploy on Azure VMs — available in 60+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-white/5 flex items-center justify-center p-2">
        <img src="/cloud-providers/azure-icon.svg" alt="Azure" className="w-full h-full object-contain" />
      </div>
    ),
  },
];

export function ProviderSelector({
  onSelectProvider,
  onClose,
}: ProviderSelectorProps) {
  const [configuredProviders, setConfiguredProviders] = useState<ProviderOption[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const { loadCredentials } = useCredentials();

  useEffect(() => {
    const loadConfiguredProviders = async () => {
      const configured: ProviderOption[] = [];
      for (const provider of providers) {
        const existing = await loadCredentials(provider.id as "aws" | "oracle" | "gcp" | "azure");
        if (existing !== null) {
          configured.push(provider);
        }
      }
      setConfiguredProviders(configured);
      setIsLoading(false);
    };
    loadConfiguredProviders();
  }, []);

  return (
    <div className="flex flex-col h-screen bg-gray-900">
      <div className="bg-gray-800 border-b border-gray-700/50 p-6">
        <div className="flex items-center gap-4">
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition"
          >
            <svg
              className="w-6 h-6 text-gray-300"
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
          </button>
          <div>
            <h1 className="text-3xl font-bold text-blue-400">
              Select Cloud Provider
            </h1>
            <p className="text-gray-300 mt-1">
              Choose which provider to deploy your VPN server on
            </p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 flex flex-col gap-4 max-w-lg mx-auto w-full">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="w-6 h-6 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
          </div>
        ) : (
          configuredProviders.map((provider) => (
            <button
              key={provider.id}
              onClick={() => onSelectProvider(provider.id)}
              className="w-full flex items-center gap-5 p-5 bg-gray-800 card-border rounded-xl hover:glow-accent-sm transition-all text-left group"
            >
              {provider.badge}
              <div className="flex-1">
                <p className="font-semibold text-white text-lg group-hover:text-blue-300 transition-colors">
                  {provider.label}
                </p>
                <p className="text-sm text-gray-400 mt-0.5">
                  {provider.description}
                </p>
              </div>
              <svg
                className="w-5 h-5 text-gray-500 group-hover:text-blue-400 transition-colors flex-shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
