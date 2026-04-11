import { useState } from "react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { ProviderIcon } from "../providers/ProviderIcon";

interface InstanceCostRowProps {
  entry: LedgerEntryWithCost;
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatUptime(hours: number): string {
  if (hours < 1) return `${Math.round(hours * 60)}m`;
  if (hours < 24) return `${hours.toFixed(1)}h`;
  return `${(hours / 24).toFixed(1)}d`;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(2)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(3)} GB`;
}

export function InstanceCostRow({ entry }: InstanceCostRowProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const isActive = entry.terminatedAt === null;
  const bytesSentGb = entry.bytesSent / 1024 ** 3;

  const hourlyComputeRate =
    entry.uptimeHours > 0 ? entry.computeCost / entry.uptimeHours : 0;
  const hourlyIpRate =
    entry.uptimeHours > 0 ? entry.ipCost / entry.uptimeHours : 0;
  const egressRatePerGb =
    bytesSentGb > 0 ? entry.egressCost / bytesSentGb : 0;
  const storageHourlyRate = entry.storageRatePerGbMonth / 730;

  return (
    <>
      <tr
        onClick={() => setIsExpanded((prev) => !prev)}
        className="border-b border-gray-700/60 hover:bg-gray-700/30 transition-colors cursor-pointer"
      >
        <td className="py-3 px-4 w-14">
          <ProviderIcon provider={entry.provider} className="w-9 h-9 shrink-0" />
        </td>
        <td className="py-3 px-4 font-mono text-xs text-gray-400">
          {entry.instanceId.length > 22
            ? `${entry.instanceId.slice(0, 22)}…`
            : entry.instanceId}
        </td>
        <td className="py-3 px-4 text-sm text-gray-300">{entry.region}</td>
        <td className="py-3 px-4 text-sm font-mono text-blue-300">
          {entry.instanceType}
        </td>
        <td className="py-3 px-4 text-sm text-gray-400">
          {formatDate(entry.launchedAt)}
        </td>
        <td className="py-3 px-4 text-sm">
          {isActive ? (
            <span className="inline-flex items-center gap-1.5 text-green-400">
              <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse inline-block" />
              Active
            </span>
          ) : (
            <span className="text-gray-400">{formatDate(entry.terminatedAt!)}</span>
          )}
        </td>
        <td className="py-3 px-4 text-sm text-gray-300">
          {formatUptime(entry.uptimeHours)}
        </td>
        <td className="py-3 px-4">
          <div className="flex items-center justify-between gap-3">
            <span className="text-sm font-semibold text-yellow-300">
              ${entry.estimatedCost.toFixed(4)}
            </span>
            <svg
              className={`w-4 h-4 text-gray-400 transition-transform duration-200 flex-shrink-0 ${
                isExpanded ? "rotate-180" : ""
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </div>
        </td>
      </tr>
      {isExpanded && (
        <tr className="border-b border-gray-700/60 bg-gray-900/40">
          <td colSpan={8} className="px-6 py-4">
            <div className="max-w-lg">
              <table className="w-full text-sm">
                <tbody>
                  <tr>
                    <td className="py-1 text-gray-400">Compute time</td>
                    <td className="py-1 text-gray-500 text-xs">
                      {entry.uptimeHours.toFixed(2)} h × ${hourlyComputeRate.toFixed(5)}/hr
                    </td>
                    <td className="py-1 text-right text-gray-300">
                      ${entry.computeCost.toFixed(4)}
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1 text-gray-400">Reserved IP</td>
                    <td className="py-1 text-gray-500 text-xs">
                      {entry.uptimeHours.toFixed(2)} h × ${hourlyIpRate.toFixed(5)}/hr
                    </td>
                    <td className="py-1 text-right text-gray-300">
                      ${entry.ipCost.toFixed(4)}
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1 text-gray-400">Data egress</td>
                    <td className="py-1 text-gray-500 text-xs">
                      {bytesSentGb.toFixed(4)} GB × ${egressRatePerGb.toFixed(4)}/GB
                    </td>
                    <td className="py-1 text-right text-gray-300">
                      ${entry.egressCost.toFixed(4)}
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1 text-gray-400">Block storage</td>
                    <td className="py-1 text-gray-500 text-xs">
                      {entry.storageGb} GB × ${storageHourlyRate.toFixed(6)}/hr
                    </td>
                    <td className="py-1 text-right text-gray-300">
                      ${entry.storageCost.toFixed(4)}
                    </td>
                  </tr>
                  <tr className="border-t border-gray-700">
                    <td className="pt-2 font-semibold text-white" colSpan={2}>
                      Total
                    </td>
                    <td className="pt-2 text-right font-bold text-yellow-300">
                      ${entry.estimatedCost.toFixed(4)}
                    </td>
                  </tr>
                </tbody>
              </table>
              <div className="mt-3 flex gap-6 text-xs text-gray-500">
                <span>Sent: {formatBytes(entry.bytesSent)}</span>
                <span>Received: {formatBytes(entry.bytesReceived)}</span>
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}
