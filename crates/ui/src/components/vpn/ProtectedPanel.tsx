import { ShieldCheck } from "lucide-react";
import { useEffect, useState } from "react";
import { ConnectedInstance, TunnelMetrics } from "../../bindings";
import { formatBytes } from "../../lib/bytes";
import { formatDuration } from "../../lib/time";
import { Button } from "../primitives/Button";
import { ServerLocation } from "../servers/ServerLocation";
import { IpAddressesCard } from "./IpAddressesCard";
import { PanelField } from "./PanelField";

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
    <div className="h-full rounded-xl bg-gray-750 border border-gray-500/40 p-4 flex flex-col items-center">
      <div className="mt-6 w-24 h-24 rounded-full bg-success-900/40 flex items-center justify-center">
        <ShieldCheck size={40} strokeWidth={1.5} className="text-success-400" />
      </div>

      <h2 className="mt-6 text-xl font-medium text-primary">You're protected</h2>
      <div className="mt-3">
        <ServerLocation
          provider={connectedInstance.provider}
          region={connectedInstance.region}
        />
      </div>

      <section className="mt-8 w-full rounded-lg bg-gray-700 p-3 flex flex-col gap-3">
        <h3 className="text-sm text-primary">Session</h3>
        <PanelField label="Downloaded" value={formatBytes(metrics?.bytesReceived ?? 0)} />
        <PanelField label="Uploaded" value={formatBytes(metrics?.bytesSent ?? 0)} />
        <PanelField label="Duration" value={formatDuration(sessionSeconds)} />
      </section>

      <div className="mt-3 w-full">
        <IpAddressesCard
          ipV4={connectedInstance.publicIpV4}
          ipV6={connectedInstance.publicIpV6}
        />
      </div>

      <div className="mt-auto w-full pt-4">
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
          className="w-full py-2 text-sm"
        >
          Disconnect from server
        </Button>
      </div>
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
