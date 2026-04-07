import { AwsRegion } from "../../types";

interface RegionCardProps {
  region: AwsRegion;
  isSelected: boolean;
  onSelect: (region: AwsRegion) => void;
}

export function RegionCard({ region, isSelected, onSelect }: RegionCardProps) {
  return (
    <div
      onClick={() => onSelect(region)}
      className={`p-4 rounded-lg cursor-pointer transition-all border ${
        isSelected
          ? "bg-blue-600/80 glow-accent text-white transform scale-102"
          : "bg-gray-800 card-border hover:glow-accent-sm text-gray-300"
      }`}
      style={{
        transformOrigin: "center",
      }}
    >
      <div className="flex items-center justify-between mb-2">
        <h4 className="font-semibold text-base">{region.name}</h4>
        {isSelected && <div className="w-3 h-3 bg-white rounded-full"></div>}
      </div>
      <p className="text-sm opacity-75 mb-1">{region.country}</p>
      <p className="text-xs opacity-60 font-mono">{region.name}</p>
    </div>
  );
}
