import { RegionGroup, AwsRegion } from "../../types";
import { RegionGroupComponent } from "./RegionGroupComponent";

interface RegionListProps {
  groupedRegions: RegionGroup[];
  selectedRegion: AwsRegion | null;
  onRegionSelect: (region: AwsRegion) => void;
  existingInstancesCount: number;
}

export function RegionList({
  groupedRegions,
  selectedRegion,
  onRegionSelect,
  existingInstancesCount,
}: RegionListProps) {
  return (
    <div className="flex-1 p-6 flex flex-col min-h-0">
      <div className="flex-1 flex flex-col min-h-0">
        <h2 className="text-xl font-semibold mb-4 text-blue-400">
          {existingInstancesCount > 0
            ? "Deploy in New Region"
            : "Available AWS Regions"}
        </h2>
        <div className="flex-1 overflow-y-auto min-h-0">
          <div className="space-y-8 p-2">
            {groupedRegions.map((group) => (
              <RegionGroupComponent
                key={group.continent}
                group={group}
                selectedRegion={selectedRegion}
                onRegionSelect={onRegionSelect}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
