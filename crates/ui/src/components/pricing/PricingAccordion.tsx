import { useState } from "react";
import { LedgerEntryWithCost } from "../../types/ledger";
import { InstanceCostRow } from "./InstanceCostRow";
import { ProviderIcon } from "../providers/ProviderIcon";

const PROVIDER_LABELS: Record<string, string> = {
  aws: "Amazon Web Services",
  azure: "Microsoft Azure",
  gcp: "Google Cloud Platform",
  oracle: "Oracle Cloud",
};


interface PricingAccordionProps {

  provider: string;

  entries: LedgerEntryWithCost[];
}

export function PricingAccordion({ provider, entries }: PricingAccordionProps) {
  const [isOpen, setIsOpen] = useState(true);
  const totalCost = entries.reduce(
    (sum, entry) => sum + entry.estimatedCost,
    0,
  );

  const sortedEntries = [...entries].sort((a, b) => {
    if (a.terminatedAt === null && b.terminatedAt !== null) return -1;
    if (a.terminatedAt !== null && b.terminatedAt === null) return 1;
    const dateA = a.terminatedAt ?? a.launchedAt;
    const dateB = b.terminatedAt ?? b.launchedAt;
    return new Date(dateB).getTime() - new Date(dateA).getTime();
  });

  return (
    <div className="bg-gray-800 rounded-lg overflow-hidden">
      {}
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        className="w-full flex items-center justify-between px-5 py-4 hover:bg-gray-700/50 transition-colors text-left"
      >
        <div className="flex items-center gap-3">
          <ProviderIcon provider={provider} className="w-6 h-6" />
          <span className="text-white font-semibold">
            {PROVIDER_LABELS[provider] ?? provider}
          </span>
          <span className="text-sm text-gray-400">
            {entries.length} instance{entries.length !== 1 ? "s" : ""}
          </span>
        </div>

        <div className="flex items-center gap-4">
          <span className="text-yellow-300 font-bold text-lg">
            ${totalCost.toFixed(4)}
          </span>
          <svg
            className={`w-5 h-5 text-gray-400 transition-transform duration-200 ${
              isOpen ? "rotate-180" : ""
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
      </button>

      {}
      {isOpen && (
        <div className="border-t border-gray-700 overflow-x-auto">
          <table className="w-full min-w-[640px]">
            <thead>
              <tr className="bg-gray-900/60 text-xs text-gray-500 uppercase tracking-wider">
                <th className="py-2 px-4 text-left font-medium">Instance ID</th>
                <th className="py-2 px-4 text-left font-medium">Region</th>
                <th className="py-2 px-4 text-left font-medium">Type</th>
                <th className="py-2 px-4 text-left font-medium">Launched</th>
                <th className="py-2 px-4 text-left font-medium">Terminated</th>
                <th className="py-2 px-4 text-left font-medium">Uptime</th>
                <th className="py-2 px-4 text-left font-medium">Est. Cost</th>
              </tr>
            </thead>
            <tbody>
              {sortedEntries.map((entry) => (
                <InstanceCostRow key={entry.instanceId} entry={entry} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
