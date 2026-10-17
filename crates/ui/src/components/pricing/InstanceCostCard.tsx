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
        className="flex w-full items-center gap-3 px-4 py-3 text-left"
      >
        <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-bd-moderate bg-bg-medium p-1.5">
          <ProviderIcon provider={entry.provider} className="size-full" />
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-0.5">
          <span className="flex min-w-0 items-center gap-2">
            <span className="truncate text-body-sm text-fg-lighter">{entry.region}</span>
            {isActive && (
              <Tag tone="success" dot>
                Active
              </Tag>
            )}
          </span>
          <span className="truncate text-caption text-fg-medium">
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
          className={`shrink-0 text-fg-medium transition-transform ${
            isExpanded ? "rotate-180" : ""
          }`}
        />
      </button>
      {isExpanded && (
        <div className="flex flex-col gap-3 bg-bg-medium px-4 py-3">
          <p className="font-mono text-caption break-all text-fg-medium">{entry.instanceId}</p>
          <CostBreakdown entry={entry} />
        </div>
      )}
    </div>
  );
}
