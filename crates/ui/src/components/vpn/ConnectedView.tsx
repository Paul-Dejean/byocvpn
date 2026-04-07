import { useState, useEffect } from "react";
import { Instance } from "../../types";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { getRegionInfo } from "../../types/regionInfo";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

interface ConnectedViewProps {
  connectedInstance: Instance;
}

export function ConnectedView({ connectedInstance }: ConnectedViewProps) {
  const { disconnectFromVpn, vpnStatus } = useVpnConnectionContext();
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const regionInfo = getRegionInfo(connectedInstance.provider, connectedInstance.region ?? "");
  const metrics = vpnStatus.metrics;

  useEffect(() => {
    const interval = setInterval(() => setElapsedSeconds(s => s + 1), 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-col h-full text-white overflow-hidden">
      <div className="flex-1 flex flex-col px-5 pt-5 pb-2 overflow-y-auto">

        {/* Location + IPs */}
        <div className="space-y-2 mb-2">
          <div className="flex items-center gap-2.5">
            <span className="text-xl leading-none">{regionInfo.flag}</span>
            <div>
              <div className="text-sm font-medium text-white leading-tight">{regionInfo.city}</div>
              <div className="text-xs text-gray-600 font-mono">{connectedInstance.region}</div>
            </div>
          </div>
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-600 font-mono">IPv4</span>
              <span className="text-xs font-mono text-gray-400">{connectedInstance.publicIpV4 || "—"}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-600 font-mono">IPv6</span>
              <span className="text-xs font-mono text-gray-400 truncate ml-6 text-right">{connectedInstance.publicIpV6 || "—"}</span>
            </div>
          </div>
        </div>

        {/* Pulsating lock + connected status — middle hero */}
        <div className="flex-1 flex items-center justify-center">
          <div className="flex flex-col items-center gap-3">
            <div className="relative flex items-center justify-center w-44 h-44">
              <div className="absolute w-44 h-44 rounded-full border border-green-400/10 animate-ping" style={{ animationDuration: "4s" }} />
              <div className="absolute w-32 h-32 rounded-full border border-green-400/15 animate-ping" style={{ animationDuration: "3s", animationDelay: "0.75s" }} />
              <div className="absolute w-24 h-24 rounded-full border border-green-400/25 animate-ping" style={{ animationDuration: "2.5s", animationDelay: "1.25s" }} />
              <div className="absolute w-20 h-20 rounded-full border border-green-400/30" style={{ boxShadow: "0 0 24px rgba(74,222,128,0.12)" }} />
              <div
                className="w-16 h-16 rounded-full bg-green-500/20 backdrop-blur-sm flex items-center justify-center relative z-10 border border-green-400/30"
                style={{ boxShadow: "0 0 32px rgba(74,222,128,0.2)" }}
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-7 w-7 text-green-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-60" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-green-400" />
              </span>
              <span className="text-xs font-mono text-green-400 uppercase tracking-widest">Connected</span>
            </div>
          </div>
        </div>

        {/* Uptime */}
        <div className="flex items-center justify-between mb-3">
          <span className="text-xs text-gray-600 uppercase tracking-widest font-mono">Uptime</span>
          <span className="text-sm font-mono font-bold text-white tabular-nums">{formatDuration(elapsedSeconds)}</span>
        </div>

        {/* Upload / Download */}
        <div className="grid grid-cols-2 gap-2">
          <div
            className="rounded-xl px-4 py-3"
            style={{ background: "rgba(74,222,128,0.06)", border: "1px solid rgba(74,222,128,0.12)" }}
          >
            <div className="flex items-center gap-1.5 mb-1.5">
              <svg className="w-3 h-3 text-green-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M7 11l5-5m0 0l5 5m-5-5v12" />
              </svg>
              <span className="text-xs text-gray-500 uppercase tracking-wide">Upload</span>
            </div>
            <div className="font-mono font-bold text-green-400 tabular-nums">
              <span className="text-xl">{formatBytes(metrics?.uploadRate ?? 0)}</span>
              <span className="text-xs text-green-700 ml-0.5">/s</span>
            </div>
            <div className="text-xs text-gray-700 font-mono mt-1">{formatBytes(metrics?.bytesSent ?? 0)} total</div>
          </div>

          <div
            className="rounded-xl px-4 py-3"
            style={{ background: "rgba(96,165,250,0.06)", border: "1px solid rgba(96,165,250,0.12)" }}
          >
            <div className="flex items-center gap-1.5 mb-1.5">
              <svg className="w-3 h-3 text-blue-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M17 13l-5 5m0 0l-5-5m5 5V6" />
              </svg>
              <span className="text-xs text-gray-500 uppercase tracking-wide">Download</span>
            </div>
            <div className="font-mono font-bold text-blue-400 tabular-nums">
              <span className="text-xl">{formatBytes(metrics?.downloadRate ?? 0)}</span>
              <span className="text-xs text-blue-700 ml-0.5">/s</span>
            </div>
            <div className="text-xs text-gray-700 font-mono mt-1">{formatBytes(metrics?.bytesReceived ?? 0)} total</div>
          </div>
        </div>

      </div>

      {/* Disconnect — pinned bottom */}
      <div className="px-5 pb-5 pt-2">
        <button
          onClick={disconnectFromVpn}
          className="w-full py-3 rounded-xl font-semibold text-white transition-all"
          style={{ background: "rgba(239,68,68,0.1)", border: "1px solid rgba(239,68,68,0.25)" }}
          onMouseEnter={e => { (e.currentTarget as HTMLButtonElement).style.background = "rgba(239,68,68,0.22)"; }}
          onMouseLeave={e => { (e.currentTarget as HTMLButtonElement).style.background = "rgba(239,68,68,0.1)"; }}
        >
          Disconnect
        </button>
      </div>
    </div>
  );
}
