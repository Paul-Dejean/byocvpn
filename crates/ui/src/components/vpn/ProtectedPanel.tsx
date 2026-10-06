import { ShieldCheck } from "lucide-react";
import { ConnectedInstance, TunnelMetrics } from "../../bindings";
import { useSessionSeconds } from "../../hooks/useSessionSeconds";
import { formatBytes } from "../../lib/bytes";
import { formatDuration } from "../../lib/time";
import { Button } from "../primitives/Button";
import { ServerLocation } from "../servers/ServerLocation";
import { IpAddressesCard } from "./IpAddressesCard";
import { PanelField } from "./PanelField";
import { StatusHero } from "./StatusHero";

interface ProtectedPanelProps {
  connectedInstance: ConnectedInstance;
  metrics: TunnelMetrics | null;
  connectedAt: number | null;
  isDisconnecting: boolean;
  isDaemonRunning: boolean;
  onDisconnect: () => void;
}

export function ProtectedPanel({
  connectedInstance,
  metrics,
  connectedAt,
  isDisconnecting,
  isDaemonRunning,
  onDisconnect,
}: ProtectedPanelProps) {
  const sessionSeconds = useSessionSeconds(connectedAt);

  return (
    <div className="h-full flex flex-col gap-3">
      <div className="flex-1 min-h-0 flex flex-col gap-3">
        <StatusHero
          tone="protected"
          icon={<ShieldCheck size={48} strokeWidth={1.5} className="text-fg-success-moderate" />}
          title="You're protected"
        >
          <ServerLocation
            provider={connectedInstance.provider}
            region={connectedInstance.region}
            layout="stacked"
          />
        </StatusHero>

        <section className="rounded-lg bg-bg-medium p-4 flex flex-col gap-3">
          <h3 className="text-body-sm text-fg-lighter">Session</h3>
          <div className="flex flex-col gap-3">
            <PanelField label="Downloaded" value={formatBytes(metrics?.bytesReceived ?? 0)} />
            <PanelField label="Uploaded" value={formatBytes(metrics?.bytesSent ?? 0)} />
            <PanelField label="Duration" value={formatDuration(sessionSeconds)} />
          </div>
        </section>

        <IpAddressesCard
          ipV4={connectedInstance.publicIpV4}
          ipV6={connectedInstance.publicIpV6}
        />
      </div>

      <Button
        variant="danger"
        size="lg"
        loading={isDisconnecting}
        disabled={!isDaemonRunning}
        disabledStyle="dim"
        onClick={onDisconnect}
        title={
          isDaemonRunning
            ? undefined
            : "VPN daemon is not running. You may need to restart your computer."
        }
        className="w-full"
      >
        Disconnect from server
      </Button>
    </div>
  );
}
