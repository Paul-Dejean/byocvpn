import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { ProviderIcon } from "../providers/ProviderIcon";
import { formatCompactDate, formatUptime } from "../../lib/time";
import { formatBytes } from "../../lib/bytes";
import { Tag } from "../primitives/Tag";

interface InstanceCostRowProps {
  entry: LedgerEntryWithCost;
}

const HOURS_PER_MONTH = 730;

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
        className="border-b border-bd-faint hover:bg-bg-light transition-colors cursor-pointer text-caption whitespace-nowrap"
      >
        <td className="py-2.5 px-3">
          <div className="w-7 h-7 rounded-md bg-bg-medium border border-bd-moderate flex items-center justify-center p-1.5">
            <ProviderIcon provider={entry.provider} className="w-full h-full" />
          </div>
        </td>
        <td className="py-2.5 px-3 font-mono text-fg-medium truncate" title={entry.instanceId}>
          {entry.instanceId}
        </td>
        <td className="py-2.5 px-3 text-fg-lighter truncate" title={entry.region}>
          {entry.region}
        </td>
        <td className="py-2.5 px-3 font-mono text-fg-medium truncate" title={entry.instanceType}>
          {entry.instanceType}
        </td>
        <td className="py-2.5 px-3 text-fg-medium tabular-nums">
          {formatCompactDate(entry.launchedAt)}
        </td>
        <td className="py-2.5 px-3 tabular-nums">
          {isActive ? (
            <Tag tone="success" dot>
              Active
            </Tag>
          ) : (
            <span className="text-fg-medium">{formatCompactDate(entry.terminatedAt ?? "")}</span>
          )}
        </td>
        <td className="py-2.5 px-3 text-fg-lighter tabular-nums">
          {formatUptime(entry.uptimeHours)}
        </td>
        <td className="py-2.5 px-3">
          <div className="flex items-center justify-between gap-2">
            {entry.isPricingUnknown ? (
              <span className="text-fg-moderate" title="Pricing unavailable for this instance type">
                —
              </span>
            ) : (
              <span className="text-fg-lighter tabular-nums">
                ${entry.estimatedCost.toFixed(4)}
              </span>
            )}
            <ChevronDown
              size={14}
              className={`text-fg-medium transition-transform flex-shrink-0 ${
                isExpanded ? "rotate-180" : ""
              }`}
            />
          </div>
        </td>
      </tr>
      {isExpanded && (
        <tr className="border-b border-bd-faint bg-bg-medium">
          <td colSpan={8} className="px-4 py-3">
            {entry.isPricingUnknown ? (
              <p className="text-caption text-fg-medium">
                Pricing is unavailable for this instance type, so no cost
                estimate can be shown.
              </p>
            ) : (
              <div className="flex flex-col gap-3">
                <dl className="grid grid-cols-[120px_1fr_auto] gap-x-6 gap-y-1.5 max-w-[520px] text-caption whitespace-nowrap">
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
                  <dt className="col-span-2 pt-1.5 border-t border-bd-faint text-fg-lighter">
                    Total
                  </dt>
                  <dd className="pt-1.5 border-t border-bd-faint text-right text-fg-lighter tabular-nums">
                    ${entry.estimatedCost.toFixed(4)}
                  </dd>
                </dl>
                <p className="text-caption text-fg-moderate">
                  Sent {formatBytes(entry.bytesSent)} · Received {formatBytes(entry.bytesReceived)}
                </p>
              </div>
            )}
          </td>
        </tr>
      )}
    </>
  );
}

interface CostLineProps {
  label: string;
  detail: string;
  amount: number;
}

function CostLine({ label, detail, amount }: CostLineProps) {
  return (
    <>
      <dt className="text-fg-medium">{label}</dt>
      <dd className="text-fg-moderate">{detail}</dd>
      <dd className="text-right text-fg-lighter tabular-nums">${amount.toFixed(4)}</dd>
    </>
  );
}
