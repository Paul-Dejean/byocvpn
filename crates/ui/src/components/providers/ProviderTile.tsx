import { CloudProviderName } from "../../types";
import { ProviderIcon } from "./ProviderIcon";

interface ProviderTileProps {
  provider: CloudProviderName;
  label: string;
  isSelected: boolean;
  onSelect: () => void;
}

export const PROVIDER_TILE_LABELS: Record<CloudProviderName, string> = {
  [CloudProviderName.Aws]: "AWS Account",
  [CloudProviderName.Oracle]: "Oracle Cloud",
  [CloudProviderName.Gcp]: "Google Cloud",
  [CloudProviderName.Azure]: "Microsoft Azure",
};

export function ProviderTile({ provider, label, isSelected, onSelect }: ProviderTileProps) {
  return (
    <button
      type="button"
      onClick={onSelect}
      aria-pressed={isSelected}
      className={`w-[124px] h-[78px] rounded-lg border p-3 flex flex-col justify-between items-start text-left transition-colors ${
        isSelected
          ? "bg-bg-medium border-bd-brand"
          : "bg-bg-medium border-bd-moderate hover:bg-bg-light"
      }`}
    >
      <ProviderIcon provider={provider} className="w-6 h-6" />
      <span className="text-caption text-fg-lighter whitespace-nowrap">{label}</span>
    </button>
  );
}
