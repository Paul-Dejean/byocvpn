import { MetricsDetails } from "../common/MetricsDisplay";
import { SettingsButton } from "../settings/SettingsButton";
import { useVpnMetrics, useVpnConnection } from "../../hooks";
import { Instance } from "../../types";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

interface ConnectedViewProps {
  connectedInstance: Instance;
  onNavigateToSettings?: () => void;
}

export function ConnectedView({
  connectedInstance,
  onNavigateToSettings,
}: ConnectedViewProps) {
  // Use hooks specific to connected view
  const { disconnectFromVpn } = useVpnConnection();
  const metrics = useVpnMetrics(true);

  const handleDisconnectClick = async () => {
    await disconnectFromVpn();
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white overflow-hidden">
      {/* Header */}
      <div className="bg-gray-800 p-6 border-b border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
              <h1 className="text-2xl font-bold text-green-400">
                Connected to VPN
              </h1>
            </div>
          </div>
          <SettingsButton onClick={() => onNavigateToSettings?.()} />
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex justify-center p-8 overflow-y-auto">
        <div className="max-w-3xl w-full space-y-6">
          {/* Compact Server Info Bar */}
          {connectedInstance && (
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <div className="flex items-center justify-between flex-wrap gap-4">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                  <span className="text-sm text-gray-400">Connected to</span>
                  <span className="font-semibold text-white">
                    {connectedInstance.region}
                  </span>
                </div>
                <div className="flex items-center gap-4 text-sm">
                  <div>
                    <span className="text-gray-400">IPv4: </span>
                    <span className="text-white font-mono">
                      {connectedInstance.publicIpV4}
                    </span>
                  </div>
                  {connectedInstance.publicIpV6 && (
                    <div>
                      <span className="text-gray-400">IPv6: </span>
                      <span className="text-white font-mono text-xs">
                        {connectedInstance.publicIpV6}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Animated Upload/Download Rates */}
          <div className="grid grid-cols-2 gap-6">
            {/* Upload Rate Card */}
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 relative overflow-hidden">
              {/* Animated background pulse */}
              <div
                className="absolute inset-0 bg-gradient-to-br from-green-500/10 to-transparent animate-pulse"
                style={{ animationDuration: "2s" }}
              ></div>

              <div className="relative z-10">
                <div className="flex items-center gap-2 mb-2">
                  <svg
                    className="w-5 h-5 text-green-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M7 11l5-5m0 0l5 5m-5-5v12"
                    />
                  </svg>
                  <span className="text-sm text-gray-400 uppercase tracking-wide">
                    Upload
                  </span>
                </div>
                <div className="text-3xl font-bold text-green-400 font-mono">
                  {formatBytes(metrics?.uploadRate ?? 0)}/s
                </div>
                <div className="mt-2 text-xs text-gray-500">
                  Total: {formatBytes(metrics?.bytesSent ?? 0)}
                </div>
              </div>

              {/* Animated bars */}
              <div className="absolute bottom-0 left-0 right-0 h-1 bg-gray-700">
                <div
                  className="h-full bg-green-500 transition-all duration-500 ease-out"
                  style={{
                    width: `${Math.min(((metrics?.uploadRate ?? 0) / (1024 * 1024)) * 100, 100)}%`,
                  }}
                ></div>
              </div>
            </div>

            {/* Download Rate Card */}
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 relative overflow-hidden">
              {/* Animated background pulse */}
              <div
                className="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-transparent animate-pulse"
                style={{ animationDuration: "2s", animationDelay: "0.5s" }}
              ></div>

              <div className="relative z-10">
                <div className="flex items-center gap-2 mb-2">
                  <svg
                    className="w-5 h-5 text-blue-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M17 13l-5 5m0 0l-5-5m5 5V6"
                    />
                  </svg>
                  <span className="text-sm text-gray-400 uppercase tracking-wide">
                    Download
                  </span>
                </div>
                <div className="text-3xl font-bold text-blue-400 font-mono">
                  {formatBytes(metrics?.downloadRate ?? 0)}/s
                </div>
                <div className="mt-2 text-xs text-gray-500">
                  Total: {formatBytes(metrics?.bytesReceived ?? 0)}
                </div>
              </div>

              {/* Animated bars */}
              <div className="absolute bottom-0 left-0 right-0 h-1 bg-gray-700">
                <div
                  className="h-full bg-blue-500 transition-all duration-500 ease-out"
                  style={{
                    width: `${Math.min(((metrics?.downloadRate ?? 0) / (1024 * 1024)) * 100, 100)}%`,
                  }}
                ></div>
              </div>
            </div>
          </div>

          {/* Detailed Metrics */}
          <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <h2 className="text-lg font-semibold mb-4 text-blue-400">
              Packet Statistics
            </h2>
            <MetricsDetails metrics={metrics} />
          </div>

          {/* Disconnect Button - update onClick handler */}
          <button
            onClick={handleDisconnectClick}
            className="px-8 py-4 bg-red-600 hover:bg-red-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl"
          >
            Disconnect
          </button>

          {/* Info */}
          <div className="text-center text-sm text-gray-400">
            <p>Your traffic is encrypted and routed through this VPN server</p>
          </div>
        </div>
      </div>
    </div>
  );
}
