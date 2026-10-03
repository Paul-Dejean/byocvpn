import { Plus, Server } from "lucide-react";
import { Button } from "../primitives/Button";

interface EmptyServersProps {
  onDeploy: () => void;
}

export function EmptyServers({ onDeploy }: EmptyServersProps) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-8">
      <div className="w-[190px] h-[190px] rounded-xl border border-dashed border-bd-moderate flex items-center justify-center">
        <Server size={72} strokeWidth={1.25} className="text-fg-moderate" />
      </div>
      <div className="flex flex-col items-center gap-2 text-center">
        <h2 className="text-body font-medium text-fg-lighter">No servers yet</h2>
        <p className="text-body-sm text-fg-medium max-w-[220px]">
          Deploy your first server to start browsing privately.
        </p>
      </div>
      <Button
        variant="primary"
        size="lg"
        onClick={onDeploy}
        icon={<Plus size={16} />}
      >
        Deploy server
      </Button>
    </div>
  );
}
