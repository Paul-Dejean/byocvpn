import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { ProviderIcon } from "../providers/ProviderIcon";
import { formatDate, formatUptime } from "../../lib/time";
import { formatBytes } from "../../lib/bytes";

interface InstanceCostRowProps {
  entry: LedgerEntryWithCost;
}

const HOURS_PER_MONTH = 730;
const INSTANCE_ID_PREVIEW_LENGTH = 22;

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
  const storageHourlyRate = entry.storageRatePerGbMonth / HOURS_PER_MONTH;

  return (
    <>
      <tr
        onClick={() => setIsExpanded((previous) => !previous)}
        className="border-b border-gray-500/40 hover:bg-gray-700/60 transition-colors cursor-pointer"
      >
        <td className="py-3 px-4 w-14">
          <div className="w-8 h-8 rounded-lg bg-gray-700 border border-gray-500/60 flex items-center justify-center p-1.5">
            <ProviderIcon provider={entry.provider} className="w-full h-full" />
          </div>
        </td>
        <td className="py-3 px-4 font-mono text-xs text-gray-300">
          {truncateInstanceId(entry.instanceId)}
        </td>
        <td className="py-3 px-4 text-sm text-gray-200">{entry.region}</td>
        <td className="py-3 px-4 text-xs font-mono text-gray-200">
          {entry.instanceType}
        </td>
        <td className="py-3 px-4 text-xs text-gray-300">
          {formatDate(entry.launchedAt)}
        </td>
        <td className="py-3 px-4 text-xs">
          {isActive ? (
            <span className="inline-flex items-center gap-1.5 text-success-300">
              <span className="w-1.5 h-1.5 rounded-full bg-success-400 inline-block" />
              Active
            </span>
          ) : (
            <span className="text-gray-300">{formatDate(entry.terminatedAt ?? "")}</span>
          )}
        </td>
        <td className="py-3 px-4 text-sm text-gray-200 tabular-nums">
          {formatUptime(entry.uptimeHours)}
        </td>
        <td className="py-3 px-4">
          <div className="flex items-center justify-between gap-3">
            {entry.isPricingUnknown ? (
              <span
                className="text-sm text-gray-400"
                title="Pricing unavailable for this instance type"
              >
                —
              </span>
            ) : (
              <span className="text-sm text-primary tabular-nums">
                ${entry.estimatedCost.toFixed(4)}
              </span>
            )}
            <ChevronDown
              size={16}
              className={`text-gray-300 transition-transform flex-shrink-0 ${
                isExpanded ? "rotate-180" : ""
              }`}
            />
          </div>
        </td>
      </tr>
      {isExpanded && (
        <tr className="border-b border-gray-500/40 bg-gray-800/60">
          <td colSpan={8} className="px-6 py-4">
            <div className="max-w-lg flex flex-col gap-3">
              {entry.isPricingUnknown ? (
                <p className="text-xs text-gray-300">
                  Pricing is unavailable for this instance type, so no cost
                  estimate can be shown.
                </p>
              ) : (
                <table className="w-full text-xs">
                  <tbody>
                    <CostLine
                      label="Compute time"
                      detail={`${entry.uptimeHours.toFixed(2)} h × $${hourlyComputeRate.toFixed(5)}/hr`}
                      amount={entry.computeCost}
                    />
                    <CostLine
                      label="Reserved IP"
                      detail={`${entry.uptimeHours.toFixed(2)} h × $${hourlyIpRate.toFixed(5)}/hr`}
                      amount={entry.ipCost}
                    />
                    <CostLine
                      label="Data egress"
                      detail={`${bytesSentGb.toFixed(4)} GB × $${egressRatePerGb.toFixed(4)}/GB`}
                      amount={entry.egressCost}
                    />
                    <CostLine
                      label="Block storage"
                      detail={`${entry.storageGb} GB × $${storageHourlyRate.toFixed(6)}/hr`}
                      amount={entry.storageCost}
                    />
                    <tr className="border-t border-gray-500/40">
                      <td className="pt-2 text-primary" colSpan={2}>
                        Total
                      </td>
                      <td className="pt-2 text-right text-primary tabular-nums">
                        ${entry.estimatedCost.toFixed(4)}
                      </td>
                    </tr>
                  </tbody>
                </table>
              )}
              <div className="flex gap-6 text-xs text-gray-400">
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

function truncateInstanceId(instanceId: string): string {
  return instanceId.length > INSTANCE_ID_PREVIEW_LENGTH
    ? `${instanceId.slice(0, INSTANCE_ID_PREVIEW_LENGTH)}…`
    : instanceId;
}

interface CostLineProps {
  label: string;
  detail: string;
  amount: number;
}

function CostLine({ label, detail, amount }: CostLineProps) {
  return (
    <tr>
      <td className="py-1 text-gray-200">{label}</td>
      <td className="py-1 text-gray-400">{detail}</td>
      <td className="py-1 text-right text-gray-200 tabular-nums">
        ${amount.toFixed(4)}
      </td>
    </tr>
  );
}
