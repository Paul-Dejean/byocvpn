import { useState } from "react";
import { ChevronRight, Clock, DollarSign } from "lucide-react";
import { Instance, InstanceState } from "../../types";
import { useInstanceUptime } from "../../hooks/useInstanceUptime";
import { useInstanceCost } from "../../hooks/useInstanceCost";
import { formatDuration } from "../../lib/time";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { Tag } from "../primitives/Tag";
import { ServerLocation } from "./ServerLocation";
import { DisconnectReminder } from "./DisconnectReminder";

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
    <div className="rounded-xl bg-bg-medium border border-bd-moderate flex flex-col gap-3">
      <div className="px-4 pt-4 flex flex-col gap-4">
        <div className="flex items-center justify-between gap-3 h-5">
          <ServerLocation provider={instance.provider} region={instance.region} />
          {isConnected && (
            <Tag tone="success" dot>
              Connected
            </Tag>
          )}
          {isInstalling && (
            <span className="flex items-center gap-1.5 text-caption text-fg-brand">
              <Spinner size="w-3 h-3" color="border-bd-brand" />
              Installing the VPN software
            </span>
          )}
          {hasError && <span className="text-caption text-fg-danger-moderate">Error</span>}
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
            className="ml-auto text-fg-medium hover:text-fg-lighter transition-colors"
          >
            <ChevronRight
              size={16}
              className={`transition-transform ${isExpanded ? "rotate-90" : ""}`}
            />
          </button>
        </div>

        {isExpanded && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-caption">
            <Detail label="Instance ID" value={instance.id} />
            <Detail label="Type" value={instance.instanceType} />
            <Detail label="IPv4" value={instance.publicIpV4} />
            <Detail label="IPv6" value={instance.publicIpV6} />
          </dl>
        )}

        {hasError && instance.errorReason && (
          <p className="text-caption text-fg-danger-moderate">{instance.errorReason}</p>
        )}

        {!isTerminating && <DisconnectReminder instanceId={instance.id} />}
      </div>

      <div className="border-t border-bd-moderate" />

      <div className="px-4 pb-3 flex items-center gap-2">
        <Button
          variant="secondary"
          size="lg"
          loading={isTerminating}
          disabledStyle="dim"
          onClick={() => onTerminate(instance)}
        >
          Terminate server
        </Button>
        {!isInstalling && (
          <Button
            variant="primary"
            size="lg"
            loading={isConnecting}
            disabled={!canConnect}
            disabledStyle="dim"
            onClick={() => onConnect(instance)}
          >
            Connect to VPN
          </Button>
        )}
      </div>
    </div>
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
      <span className="text-caption text-fg-medium">{label}</span>
      <span className="flex items-center gap-1.5 text-feature leading-[26px] text-fg-lighter tabular-nums">
        <span className="text-fg-medium">{icon}</span>
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
      <dt className="text-fg-medium">{label}</dt>
      <dd className="text-fg-lighter truncate">{value || "—"}</dd>
    </>
  );
}
