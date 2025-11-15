import { ExistingInstance, ServerDetails } from "../../types";
import { InstanceCard } from "./InstanceCard";

interface InstanceListProps {
  instances: ExistingInstance[];
  selectedInstance: ServerDetails | null;
  onInstanceSelect: (instance: ExistingInstance) => void;
}

export function InstanceList({
  instances,
  selectedInstance,
  onInstanceSelect,
}: InstanceListProps) {
  if (instances.length === 0) {
    return null;
  }

  return (
    <div className="flex-1 p-6 flex flex-col min-h-0">
      <div className="flex-1 flex flex-col min-h-0">
        <h2 className="text-xl font-semibold mb-4 text-blue-400">
          Existing VPN Servers
        </h2>
        <div className="flex-1 overflow-y-auto min-h-0">
          <div className="grid grid-cols-1 gap-6 p-2">
            {instances.map((instance) => (
              <InstanceCard
                key={instance.id}
                instance={instance}
                isSelected={selectedInstance?.instance_id === instance.id}
                onSelect={onInstanceSelect}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
