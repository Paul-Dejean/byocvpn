import { ProviderIcon } from "../providers/ProviderIcon";

const PROVIDER_SHORT_LABELS: Record<string, string> = {
  aws: "AWS",
  azure: "Azure",
  gcp: "GCP",
  oracle: "Oracle",
};

interface ProviderFilterProps {
  availableProviders: string[];
  selectedProvider: string | null;
  onSelectProvider: (provider: string | null) => void;
}

export function ProviderFilter({
  availableProviders,
  selectedProvider,
  onSelectProvider,
}: ProviderFilterProps) {
  return (
    <div className="flex items-center gap-2 flex-wrap">
      <button
        onClick={() => onSelectProvider(null)}
        className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
          selectedProvider === null
            ? "bg-blue-500 text-white"
            : "bg-gray-700 text-gray-300 hover:bg-gray-600"
        }`}
      >
        All
      </button>
      {availableProviders.map((provider) => (
        <button
          key={provider}
          onClick={() => onSelectProvider(provider)}
          className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-sm font-medium transition-colors ${
            selectedProvider === provider
              ? "bg-blue-500 text-white"
              : "bg-gray-700 text-gray-300 hover:bg-gray-600"
          }`}
        >
          <ProviderIcon provider={provider} className="w-4 h-4" />
          {PROVIDER_SHORT_LABELS[provider] ?? provider}
        </button>
      ))}
    </div>
  );
}
