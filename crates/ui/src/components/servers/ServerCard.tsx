import { useState } from "react";
import { ChevronRight, Clock, DollarSign } from "lucide-react";
import { Instance, InstanceState } from "../../types";
import { useInstanceUptime } from "../../hooks/useInstanceUptime";
import { useInstanceCost } from "../../hooks/useInstanceCost";
import { formatDuration } from "../../lib/time";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { ServerLocation } from "./ServerLocation";

interface ServerCardProps {
  instance: Instance;
  isConnected: boolean;
  isConnecting: boolean;
  isTerminating: boolean;
  onConnect: (instance: Instance) => void;
  onTerminate: (instance: Instance) => void;
}

export function ServerCard({
  instance,
  isConnected,
  isConnecting,
  isTerminating,
  onConnect,
  onTerminate,
}: ServerCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const uptimeSeconds = useInstanceUptime(instance.launchedAt);
  const estimatedCost = useInstanceCost(
    instance.provider,
    instance.instanceType,
    uptimeSeconds,
  );

  const isInstalling = instance.state === InstanceState.Installing;
  const hasError = instance.state === InstanceState.Error;
  const canConnect = instance.state === InstanceState.Running && !isConnected;

  return (
    <div className="rounded-xl bg-gray-750 border border-gray-500/50">
      <div className="p-3 flex flex-col gap-3">
        <div className="flex items-center justify-between gap-3">
          <ServerLocation provider={instance.provider} region={instance.region} />
          {isConnected && <ConnectedBadge />}
          {isInstalling && (
            <span className="flex items-center gap-1.5 text-xs text-blue-300">
              <Spinner size="w-3 h-3" color="border-blue-300" />
              Installing
            </span>
          )}
          {hasError && (
            <span className="text-xs text-danger-400">Error</span>
          )}
        </div>

        <div className="flex items-center gap-8">
          <Metric
            label="Duration"
            icon={<Clock size={16} />}
            value={formatDuration(uptimeSeconds)}
          />
          <Metric
            label="Cost"
            icon={<DollarSign size={16} />}
            value={estimatedCost === null ? "—" : estimatedCost.toFixed(4)}
          />
          <button
            type="button"
            onClick={() => setIsExpanded((previous) => !previous)}
            aria-label={isExpanded ? "Hide details" : "Show details"}
            className="ml-auto text-gray-300 hover:text-primary transition-colors"
          >
            <ChevronRight
              size={18}
              className={`transition-transform ${isExpanded ? "rotate-90" : ""}`}
            />
          </button>
        </div>

        {isExpanded && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-xs">
            <Detail label="Instance ID" value={instance.id} />
            <Detail label="Type" value={instance.instanceType} />
            <Detail label="IPv4" value={instance.publicIpV4} />
            <Detail label="IPv6" value={instance.publicIpV6} />
          </dl>
        )}

        {hasError && instance.errorReason && (
          <p className="text-xs text-danger-300">{instance.errorReason}</p>
        )}
      </div>

      <div className="border-t border-gray-500/50 p-3 flex items-center gap-2">
        <Button
          variant="secondary"
          size="none"
          loading={isTerminating}
          disabledStyle="dim"
          onClick={() => onTerminate(instance)}
          className="px-3 py-1.5 text-sm"
        >
          Terminate server
        </Button>
        {!isInstalling && (
          <Button
            variant="primary"
            size="none"
            loading={isConnecting}
            disabled={!canConnect}
            disabledStyle="dim"
            onClick={() => onConnect(instance)}
            className="px-3 py-1.5 text-sm"
          >
            Connect to VPN
          </Button>
        )}
      </div>
    </div>
  );
}

function ConnectedBadge() {
  return (
    <span className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-success-900/50 text-xs text-success-300">
      <span className="w-1.5 h-1.5 rounded-full bg-success-400" />
      Connected
    </span>
  );
}

interface MetricProps {
  label: string;
  icon: React.ReactNode;
  value: string;
}

function Metric({ label, icon, value }: MetricProps) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs text-gray-300">{label}</span>
      <span className="flex items-center gap-1.5 text-lg text-primary tabular-nums">
        <span className="text-gray-300">{icon}</span>
        {value}
      </span>
    </div>
  );
}

interface DetailProps {
  label: string;
  value: string;
}

function Detail({ label, value }: DetailProps) {
  return (
    <>
      <dt className="text-gray-300">{label}</dt>
      <dd className="text-primary font-mono truncate">{value || "—"}</dd>
    </>
  );
}
