import { ProviderIcon } from "../providers/ProviderIcon";
import { CloudProviderName } from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { FilterChip } from "../primitives/FilterChip";

interface ProviderFilterProps {
  availableProviders: CloudProviderName[];
  selectedProvider: CloudProviderName | null;
  onSelectProvider: (provider: CloudProviderName | null) => void;
}

export function ProviderFilter({
  availableProviders,
  selectedProvider,
  onSelectProvider,
}: ProviderFilterProps) {
  return (
    <div className="flex items-center gap-2 flex-wrap">
      <FilterChip selected={selectedProvider === null} onClick={() => onSelectProvider(null)}>
        All
      </FilterChip>
      {availableProviders.map((provider) => (
        <FilterChip
          key={provider}
          selected={selectedProvider === provider}
          onClick={() => onSelectProvider(provider)}
        >
          <ProviderIcon provider={provider} className="w-4 h-4" />
          {PROVIDER_METADATA[provider].shortLabel}
        </FilterChip>
      ))}
    </div>
  );
}
