import { ShieldCheck } from "lucide-react";
import { useEffect, useState } from "react";
import { ConnectedInstance, TunnelMetrics } from "../../bindings";
import { formatBytes } from "../../lib/bytes";
import { formatDuration } from "../../lib/time";
import { Button } from "../primitives/Button";
import { ServerLocation } from "../servers/ServerLocation";
import { IpAddressesCard } from "./IpAddressesCard";
import { PanelField } from "./PanelField";
import { StatusHero } from "./StatusHero";

const TICK_INTERVAL_MS = 1000;

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
          icon={<ShieldCheck size={48} strokeWidth={1.5} className="text-success-400" />}
          title="You're protected"
        >
          <ServerLocation
            provider={connectedInstance.provider}
            region={connectedInstance.region}
          />
        </StatusHero>

        <section className="rounded-lg bg-gray-700 p-4 flex flex-col gap-3">
          <h3 className="text-sm text-primary">Session</h3>
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
        size="none"
        loading={isDisconnecting}
        disabled={!isDaemonRunning}
        disabledStyle="dim"
        onClick={onDisconnect}
        title={
          isDaemonRunning
            ? undefined
            : "VPN daemon is not running. You may need to restart your computer."
        }
        className="w-full h-7 text-sm"
      >
        Disconnect from server
      </Button>
    </div>
  );
}

function useSessionSeconds(connectedAt: number | null): number {
  const [fallbackStartTime] = useState(() => Date.now());
  const startTime = connectedAt !== null ? connectedAt * 1000 : fallbackStartTime;
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  useEffect(() => {
    function updateElapsedSeconds() {
      setElapsedSeconds(Math.max(0, Math.floor((Date.now() - startTime) / 1000)));
    }
    updateElapsedSeconds();
    const interval = setInterval(updateElapsedSeconds, TICK_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [startTime]);

  return elapsedSeconds;
}
