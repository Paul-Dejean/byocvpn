import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { ProviderIcon } from "../providers/ProviderIcon";
import { formatCompactDate, formatUptime } from "../../lib/time";
import { Tag } from "../primitives/Tag";
import { CostBreakdown } from "./CostBreakdown";

interface InstanceCostRowProps {
  entry: LedgerEntryWithCost;
}

export function InstanceCostRow({ entry }: InstanceCostRowProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const isActive = entry.terminatedAt === null;

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
            <CostBreakdown entry={entry} />
          </td>
        </tr>
      )}
    </>
  );
}
