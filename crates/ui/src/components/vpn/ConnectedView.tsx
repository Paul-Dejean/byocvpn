import { useState, useEffect } from "react";
import { Instance } from "../../types";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { Alert } from "../primitives/Alert";
import { Button } from "../primitives/Button";
import { formatBytes } from "../../lib/bytes";
import { formatDuration } from "../../lib/time";

interface ConnectedViewProps {
  connectedInstance: Instance;
}

export function ConnectedView({ connectedInstance }: ConnectedViewProps) {
  const { disconnectFromVpn, vpnStatus, isDaemonRunning, isDisconnecting } = useVpnConnectionContext();
  const [startTime] = useState(() => Date.now());
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const regionInfo = getRegionInfo(
    connectedInstance.provider,
    connectedInstance.region ?? "",
  );
  const metrics = vpnStatus.metrics;

  useEffect(() => {
    const interval = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);
    return () => clearInterval(interval);
  }, [startTime]);

  const connectionError = isDisconnecting ? null : vpnStatus.connectionError;

  return (
    <div className="flex flex-col h-full text-primary overflow-hidden">
      {connectionError && (
        <div className="px-4 pt-4">
          <Alert
            variant="error"
            icon={
              <svg
                className="w-4 h-4 text-danger-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"
                />
              </svg>
            }
          >
            <span className="text-xs leading-relaxed">{connectionError}</span>
          </Alert>
        </div>
      )}
      <div className="flex-1 flex flex-col p-6 pb-2 overflow-y-auto">
        {/* Location + IPs */}
        <div className="space-y-2 mb-2">
          <div className="flex items-center gap-2.5">
            <FlagIcon
              countryCode={regionInfo.countryCode}
              className="text-xl"
            />
            <div>
              <div className="text-sm font-medium text-primary leading-tight">
                {regionInfo.city}
              </div>
              <div className="text-xs text-gray-400 font-mono">
                {connectedInstance.region}
              </div>
            </div>
          </div>
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-400 font-mono">IPv4</span>
              <span className="text-xs font-mono text-gray-400">
                {connectedInstance.publicIpV4 || "—"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-400 font-mono">IPv6</span>
              <span className="text-xs font-mono text-gray-400 truncate ml-6 text-right">
                {connectedInstance.publicIpV6 || "—"}
              </span>
            </div>
          </div>
        </div>

        {/* Pulsating lock + connected status — middle hero */}
        <div className="flex-1 flex items-center justify-center">
          <div className="flex flex-col items-center gap-3">
            <div className="relative flex items-center justify-center w-44 h-44">
              <div className="absolute w-44 h-44 rounded-full border border-success-400/10 animate-ping [animation-duration:4s]" />
              <div className="absolute w-32 h-32 rounded-full border border-success-400/15 animate-ping [animation-duration:3s] [animation-delay:0.75s]" />
              <div className="absolute w-24 h-24 rounded-full border border-success-400/25 animate-ping [animation-duration:2.5s] [animation-delay:1.25s]" />
              <div className="absolute w-20 h-20 rounded-full border border-success-400/30 glow-green-ring" />
              <div className="w-16 h-16 rounded-full bg-success-500/20 backdrop-blur-sm flex items-center justify-center relative z-10 border border-success-400/30 glow-green-core">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-7 w-7 text-success-300"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  />
                </svg>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-success-400 opacity-60" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-success-400" />
              </span>
              <span className="text-xs font-mono text-success-400 uppercase tracking-widest">
                Connected
              </span>
            </div>
          </div>
        </div>

        {/* Uptime */}
        <div className="flex items-center justify-between mb-3">
          <span className="text-xs text-gray-400 uppercase tracking-widest font-mono">
            Uptime
          </span>
          <span className="text-sm font-mono font-bold text-primary tabular-nums">
            {formatDuration(elapsedSeconds)}
          </span>
        </div>

        {/* Upload / Download */}
        <div className="grid grid-cols-2 gap-2">
          <div className="rounded-lg px-4 py-3 bg-success-400/6 border border-success-400/12">
            <div className="flex items-center gap-1.5 mb-1.5">
              <svg
                className="w-3 h-3 text-success-500 flex-shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2.5}
                  d="M7 11l5-5m0 0l5 5m-5-5v12"
                />
              </svg>
              <span className="text-xs text-gray-500 uppercase tracking-wide">
                Upload
              </span>
            </div>
            <div className="font-mono font-bold text-success-400 tabular-nums">
              <span className="text-xl">
                {formatBytes(metrics?.uploadRate ?? 0)}
              </span>
              <span className="text-xs text-success-600 ml-0.5">/s</span>
            </div>
            <div className="text-xs text-gray-500 font-mono mt-1">
              {formatBytes(metrics?.bytesSent ?? 0)} total
            </div>
          </div>

          <div className="rounded-lg px-4 py-3 bg-blue-400/6 border border-blue-400/12">
            <div className="flex items-center gap-1.5 mb-1.5">
              <svg
                className="w-3 h-3 text-blue-500 flex-shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2.5}
                  d="M17 13l-5 5m0 0l-5-5m5 5V6"
                />
              </svg>
              <span className="text-xs text-gray-500 uppercase tracking-wide">
                Download
              </span>
            </div>
            <div className="font-mono font-bold text-blue-400 tabular-nums">
              <span className="text-xl">
                {formatBytes(metrics?.downloadRate ?? 0)}
              </span>
              <span className="text-xs text-blue-600 ml-0.5">/s</span>
            </div>
            <div className="text-xs text-gray-500 font-mono mt-1">
              {formatBytes(metrics?.bytesReceived ?? 0)} total
            </div>
          </div>
        </div>
      </div>

      {/* Disconnect — pinned bottom */}
      <div className="px-6 pb-6 pt-2">
        <div className="relative group">
          <Button
            variant="ghostDanger"
            size="none"
            disabledStyle="dim"
            onClick={disconnectFromVpn}
            disabled={!isDaemonRunning}
            className="w-full py-4 text-lg"
          >
            Disconnect
          </Button>
          {!isDaemonRunning && (
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden group-hover:block w-56 text-center">
              <div className="bg-gray-800 border border-gray-600 text-gray-200 text-xs rounded-lg px-3 py-2 leading-relaxed shadow-lg">
                VPN daemon is not running. You may need to restart your computer.
              </div>
              <div className="w-2 h-2 bg-gray-800 border-r border-b border-gray-600 rotate-45 mx-auto -mt-1" />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
