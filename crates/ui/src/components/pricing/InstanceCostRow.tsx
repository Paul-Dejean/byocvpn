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
        className="cursor-pointer border-b border-bd-faint text-caption whitespace-nowrap transition-colors hover:bg-bg-light"
      >
        <td className="px-3 py-2.5">
          <div className="flex size-7 items-center justify-center rounded-md border border-bd-moderate bg-bg-medium p-1.5">
            <ProviderIcon provider={entry.provider} className="size-full" />
          </div>
        </td>
        <td className="truncate px-3 py-2.5 font-mono text-fg-medium" title={entry.instanceId}>
          {entry.instanceId}
        </td>
        <td className="truncate px-3 py-2.5 text-fg-lighter" title={entry.region}>
          {entry.region}
        </td>
        <td className="truncate px-3 py-2.5 font-mono text-fg-medium" title={entry.instanceType}>
          {entry.instanceType}
        </td>
        <td className="px-3 py-2.5 text-fg-medium tabular-nums">
          {formatCompactDate(entry.launchedAt)}
        </td>
        <td className="px-3 py-2.5 tabular-nums">
          {isActive ? (
            <Tag tone="success" dot>
              Active
            </Tag>
          ) : (
            <span className="text-fg-medium">{formatCompactDate(entry.terminatedAt ?? "")}</span>
          )}
        </td>
        <td className="px-3 py-2.5 text-fg-lighter tabular-nums">
          {formatUptime(entry.uptimeHours)}
        </td>
        <td className="px-3 py-2.5">
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
              className={`shrink-0 text-fg-medium transition-transform ${
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
