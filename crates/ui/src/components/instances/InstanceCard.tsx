import { Instance } from "../../types";

interface InstanceCardProps {
  instance: Instance;
  isSelected: boolean;
  onSelect: (instance: Instance) => void;
}

export function InstanceCard({
  instance,
  isSelected,
  onSelect,
}: InstanceCardProps) {
  return (
    <div
      onClick={() => onSelect(instance)}
      className={`p-4 rounded-lg cursor-pointer transition-all border ${
        isSelected
          ? "bg-blue-600/80 glow-accent text-white transform scale-102"
          : "bg-gray-800 card-border hover:glow-accent-sm"
      }`}
      style={{
        transformOrigin: "center",
      }}
    >
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-semibold text-lg">
          {instance.name || "VPN Server"}
        </h3>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 bg-green-500 rounded-full"></div>
          <span className="text-sm text-green-400">Running</span>
          {isSelected && (
            <div className="w-3 h-3 bg-white rounded-full ml-2"></div>
          )}
        </div>
      </div>
      <p className="text-sm text-gray-400 mb-1">
        <span className="font-medium">Instance ID:</span> {instance.id}
      </p>
      <p className="text-sm text-gray-400 mb-1">
        <span className="font-medium">Public IP:</span> {instance.publicIpV4}
      </p>
      <p className="text-sm text-gray-400">
        <span className="font-medium">Region:</span> {instance.region}
      </p>
    </div>
  );
}
