import React from "react";

/**
 * Full-screen step for selecting a cloud provider before region selection.
 */
interface ProviderSelectorProps {
  /** Called when the user picks a provider */
  onSelectProvider: (provider: string) => void;
  /** Called when the user presses the back button */
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
      <div className="w-14 h-14 rounded-xl bg-orange-500/20 flex items-center justify-center">
        <span className="text-orange-400 font-black text-xl">AWS</span>
      </div>
    ),
  },
  {
    id: "oracle",
    label: "Oracle Cloud Infrastructure",
    description: "Deploy on OCI Compute — includes an Always Free tier",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-red-700/30 flex items-center justify-center">
        <span className="text-red-400 font-black text-xl">OCI</span>
      </div>
    ),
  },
  {
    id: "gcp",
    label: "Google Cloud Platform",
    description:
      "Deploy on GCP Compute Engine — available in 40+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-blue-600/20 flex items-center justify-center">
        <span className="text-blue-400 font-black text-xl">GCP</span>
      </div>
    ),
  },
];

export function ProviderSelector({
  onSelectProvider,
  onClose,
}: ProviderSelectorProps) {
  return (
    <div className="flex flex-col h-screen bg-gray-900">
      {/* Header */}
      <div className="bg-gray-800 p-6 border-b border-gray-700">
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

      {/* Provider Cards */}
      <div className="flex-1 p-6 flex flex-col gap-4 justify-center max-w-lg mx-auto w-full">
        {providers.map((provider) => (
          <button
            key={provider.id}
            onClick={() => onSelectProvider(provider.id)}
            className="w-full flex items-center gap-5 p-5 bg-gray-800 border border-gray-700 rounded-xl hover:border-blue-500 hover:bg-gray-750 transition-all text-left group"
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
        ))}
      </div>
    </div>
  );
}
