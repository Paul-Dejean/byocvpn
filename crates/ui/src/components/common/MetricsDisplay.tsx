import { VpnMetrics } from "../../hooks/useVpnMetrics";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

interface MetricsDisplayProps {
  metrics: VpnMetrics | null;
  isConnected: boolean;
}

export function MetricsDisplay({ metrics, isConnected }: MetricsDisplayProps) {
  if (!isConnected) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h3 className="text-lg font-semibold text-white mb-3">
          📊 VPN Statistics
        </h3>
        <p className="text-gray-400 text-sm">Connect to view live statistics</p>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h3 className="text-lg font-semibold text-white mb-3">
          📊 VPN Statistics
        </h3>
        <p className="text-gray-400 text-sm">Loading metrics...</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="bg-gray-700 rounded-lg p-3 h-16 flex items-center">
        <div className="flex items-center justify-between w-full">
          <span className="text-gray-400 text-sm">Upload</span>
          <span className="text-green-400 font-mono text-lg font-semibold min-w-[120px] text-right">
            ↑ {formatBytes(metrics.uploadRate)}/s
          </span>
        </div>
      </div>

      <div className="bg-gray-700 rounded-lg p-3 h-16 flex items-center">
        <div className="flex items-center justify-between w-full">
          <span className="text-gray-400 text-sm">Download</span>
          <span className="text-blue-400 font-mono text-lg font-semibold min-w-[120px] text-right">
            ↓ {formatBytes(metrics.downloadRate)}/s
          </span>
        </div>
      </div>
    </div>
  );
}

// Additional metrics component for details below buttons
export function MetricsDetails({ metrics }: { metrics: VpnMetrics | null }) {
  if (!metrics) return null;

  return (
    <>
      {/* Total Data Transferred */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-gray-300 mb-3">
          Total Data Transferred
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <div className="bg-gray-700 rounded-lg p-3">
            <span className="text-gray-400 text-xs block mb-1">Upload</span>
            <span className="text-white font-mono text-sm">
              {formatBytes(metrics.bytesSent)}
            </span>
          </div>

          <div className="bg-gray-700 rounded-lg p-3">
            <span className="text-gray-400 text-xs block mb-1">Download</span>
            <span className="text-white font-mono text-sm">
              {formatBytes(metrics.bytesReceived)}
            </span>
          </div>
        </div>
      </div>

      {/* Packet Stats */}
      <div>
        <h4 className="text-sm font-medium text-gray-300 mb-3">
          Packet Statistics
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <div className="bg-gray-700 rounded-lg p-3">
            <span className="text-gray-400 text-xs block mb-1">Sent</span>
            <span className="text-white font-mono text-sm">
              {metrics.packetsSent.toLocaleString()}
            </span>
          </div>

          <div className="bg-gray-700 rounded-lg p-3">
            <span className="text-gray-400 text-xs block mb-1">Received</span>
            <span className="text-white font-mono text-sm">
              {metrics.packetsReceived.toLocaleString()}
            </span>
          </div>
        </div>
      </div>
    </>
  );
}
