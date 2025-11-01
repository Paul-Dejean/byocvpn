import { RegionGroup, AwsRegion } from "../../types";
import { RegionCard } from "./RegionCard";

interface RegionGroupComponentProps {
  group: RegionGroup;
  selectedRegion: AwsRegion | null;
  onRegionSelect: (region: AwsRegion) => void;
}

export function RegionGroupComponent({
  group,
  selectedRegion,
  onRegionSelect,
}: RegionGroupComponentProps) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold text-blue-300 border-b border-gray-700 pb-2">
        {group.continent}
      </h3>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {group.regions.map((region) => (
          <RegionCard
            key={region.name}
            region={region}
            isSelected={selectedRegion?.name === region.name}
            onSelect={onRegionSelect}
          />
        ))}
      </div>
    </div>
  );
}
