import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { ProviderIcon } from "../providers/ProviderIcon";
import { formatCompactDate, formatUptime } from "../../lib/time";
import { Tag } from "../primitives/Tag";
import { CostBreakdown } from "./CostBreakdown";

interface InstanceCostCardProps {
  entry: LedgerEntryWithCost;
}

export function InstanceCostCard({ entry }: InstanceCostCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const isActive = entry.terminatedAt === null;

  return (
    <div className="border-b border-bd-faint last:border-b-0">
      <button
        type="button"
        onClick={() => setIsExpanded((previous) => !previous)}
        aria-expanded={isExpanded}
        className="w-full px-4 py-3 flex items-center gap-3 text-left"
      >
        <div className="w-8 h-8 flex-shrink-0 rounded-md bg-bg-medium border border-bd-moderate flex items-center justify-center p-1.5">
          <ProviderIcon provider={entry.provider} className="w-full h-full" />
        </div>
        <div className="flex-1 min-w-0 flex flex-col gap-0.5">
          <span className="flex items-center gap-2 min-w-0">
            <span className="text-body-sm text-fg-lighter truncate">{entry.region}</span>
            {isActive && (
              <Tag tone="success" dot>
                Active
              </Tag>
            )}
          </span>
          <span className="text-caption text-fg-medium truncate">
            {entry.instanceType} · {formatCompactDate(entry.launchedAt)} ·{" "}
            {formatUptime(entry.uptimeHours)}
          </span>
        </div>
        {entry.isPricingUnknown ? (
          <span className="text-body-sm text-fg-moderate">—</span>
        ) : (
          <span className="text-body-sm text-fg-lighter tabular-nums">
            ${entry.estimatedCost.toFixed(4)}
          </span>
        )}
        <ChevronDown
          size={14}
          className={`text-fg-medium transition-transform flex-shrink-0 ${
            isExpanded ? "rotate-180" : ""
          }`}
        />
      </button>
      {isExpanded && (
        <div className="px-4 py-3 bg-bg-medium flex flex-col gap-3">
          <p className="text-caption font-mono text-fg-medium break-all">{entry.instanceId}</p>
          <CostBreakdown entry={entry} />
        </div>
      )}
    </div>
  );
}
