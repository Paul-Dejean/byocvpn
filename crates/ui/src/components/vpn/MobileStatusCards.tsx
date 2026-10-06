import { ShieldCheck, ShieldOff } from "lucide-react";
import { ConnectedInstance, TunnelMetrics } from "../../bindings";
import { useNetworkLocation } from "../../hooks/useNetworkLocation";
import { useSessionSeconds } from "../../hooks/useSessionSeconds";
import { formatBytes } from "../../lib/bytes";
import { formatDuration } from "../../lib/time";
import { FlagIcon } from "../FlagIcon";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import { ServerLocation } from "../servers/ServerLocation";
import { PanelField } from "./PanelField";

interface ProtectedStatusCardProps {
  connectedInstance: ConnectedInstance;
  metrics: TunnelMetrics | null;
  connectedAt: number | null;
  isDisconnecting: boolean;
  isDaemonRunning: boolean;
  onDisconnect: () => void;
}

export function ProtectedStatusCard({
  connectedInstance,
  metrics,
  connectedAt,
  isDisconnecting,
  isDaemonRunning,
  onDisconnect,
}: ProtectedStatusCardProps) {
  const sessionSeconds = useSessionSeconds(connectedAt);

  return (
    <section className="rounded-xl bg-bg-bolder border border-bd-success p-4 flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <StatusIcon isProtected />
        <div className="min-w-0 flex flex-col gap-1">
          <h2 className="text-feature font-semibold text-fg-lighter">You're protected</h2>
          <ServerLocation
            provider={connectedInstance.provider}
            region={connectedInstance.region}
          />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-3">
        <PanelField label="Downloaded" value={formatBytes(metrics?.bytesReceived ?? 0)} />
        <PanelField label="Uploaded" value={formatBytes(metrics?.bytesSent ?? 0)} />
        <PanelField label="Duration" value={formatDuration(sessionSeconds)} />
      </div>

      <PanelField label="Public IPv4" value={connectedInstance.publicIpV4 || "—"} wrap />

      <Button
        variant="danger"
        size="lg"
        loading={isDisconnecting}
        disabled={!isDaemonRunning}
        disabledStyle="dim"
        onClick={onDisconnect}
        className="w-full"
      >
        Disconnect
      </Button>
    </section>
  );
}

interface UnprotectedStatusCardProps {
  hasServers: boolean;
}

export function UnprotectedStatusCard({ hasServers }: UnprotectedStatusCardProps) {
  const { location, isLoading } = useNetworkLocation(false);
  const hasLocation = location !== null && (location.country || location.city);

  return (
    <section className="rounded-xl bg-bg-bolder border border-bd-danger p-4 flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <StatusIcon isProtected={false} />
        <div className="min-w-0 flex flex-col gap-1">
          <h2 className="text-feature font-semibold text-fg-lighter">Unprotected</h2>
          <p className="text-caption text-fg-medium">
            {hasServers
              ? "Connect to a server to go private."
              : "Add a server to stay private."}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1 min-w-0">
          <span className="text-caption text-fg-medium">Current location</span>
          {isLoading ? (
            <Spinner size="w-4 h-4" color="border-bd-strong" />
          ) : hasLocation && location ? (
            <span className="flex items-center gap-2 text-body-sm text-fg-lighter min-w-0">
              <FlagIcon countryCode={location.countryCode} />
              <span className="truncate">{location.city || location.country}</span>
            </span>
          ) : (
            <span className="text-body-sm text-fg-moderate">Unavailable</span>
          )}
        </div>
        <PanelField label="Public IPv4" value={location?.publicIpV4 || "—"} wrap />
      </div>
    </section>
  );
}

interface StatusIconProps {
  isProtected: boolean;
}

function StatusIcon({ isProtected }: StatusIconProps) {
  return (
    <div
      className={`w-12 h-12 flex-shrink-0 rounded-full flex items-center justify-center ${
        isProtected ? "bg-bg-success-faint" : "bg-bg-danger-faint"
      }`}
    >
      {isProtected ? (
        <ShieldCheck size={24} strokeWidth={1.5} className="text-fg-success-moderate" />
      ) : (
        <ShieldOff size={24} strokeWidth={1.5} className="text-fg-danger-moderate" />
      )}
    </div>
  );
}
