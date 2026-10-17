import { CloudProviderName } from "../../../types";
import { ProviderIcon } from "../ProviderIcon";

interface ProviderTileProps {
  provider: CloudProviderName;
  label: string;
  isSelected: boolean;
  onSelect: () => void;
}

export function ProviderTile({ provider, label, isSelected, onSelect }: ProviderTileProps) {
  return (
    <button
      type="button"
      onClick={onSelect}
      aria-pressed={isSelected}
      className={`flex h-[78px] w-31 flex-col items-start justify-between rounded-lg border p-3 text-left transition-colors ${
        isSelected
          ? "border-bd-brand bg-bg-medium"
          : "border-bd-moderate bg-bg-medium hover:bg-bg-light"
      }`}
    >
      <ProviderIcon provider={provider} className="size-6" />
      <span className="text-caption whitespace-nowrap text-fg-lighter">{label}</span>
    </button>
  );
}
