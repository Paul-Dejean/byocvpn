import { ExistingInstance } from "../../types";

interface InstanceCardProps {
  instance: ExistingInstance;
  isSelected: boolean;
  onSelect: (instance: ExistingInstance) => void;
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
          ? "bg-blue-600 border-blue-500 text-white transform scale-102 shadow-lg"
          : "bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600"
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
        <span className="font-medium">Public IP:</span> {instance.public_ip_v4}
      </p>
      <p className="text-sm text-gray-400">
        <span className="font-medium">Region:</span> {instance.region}
      </p>
    </div>
  );
}
