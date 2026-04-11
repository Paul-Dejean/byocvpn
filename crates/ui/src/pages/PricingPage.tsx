import { useMemo, useState } from "react";
import { useLedger } from "../hooks/useLedger";
import { SelectedMonth } from "../components/pricing/MonthFilter";
import { ProviderFilter } from "../components/pricing/ProviderFilter";
import { InstanceCostRow } from "../components/pricing/InstanceCostRow";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { EmptyState } from "../components/common/EmptyState";
import { LedgerEntryWithCost } from "../types/ledger";

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

function getCurrentMonth(): SelectedMonth {
  const now = new Date();
  return { year: now.getFullYear(), month: now.getMonth() + 1 };
}

function monthKeyFromEntry(entry: LedgerEntryWithCost): string {
  const date = new Date(entry.launchedAt);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function parseMonthKey(key: string): SelectedMonth {
  const [year, month] = key.split("-").map(Number);
  return { year, month };
}

function formatMonthKey(month: SelectedMonth): string {
  return `${month.year}-${String(month.month).padStart(2, "0")}`;
}

function sortEntries(entries: LedgerEntryWithCost[]): LedgerEntryWithCost[] {
  return [...entries].sort((entryA, entryB) => {
    if (entryA.terminatedAt === null && entryB.terminatedAt !== null) return -1;
    if (entryA.terminatedAt !== null && entryB.terminatedAt === null) return 1;
    const dateA = entryA.terminatedAt ?? entryA.launchedAt;
    const dateB = entryB.terminatedAt ?? entryB.launchedAt;
    return new Date(dateB).getTime() - new Date(dateA).getTime();
  });
}

export function PricingPage() {
  const { entries, isLoading, error, refetch } = useLedger();

  const availableMonths = useMemo<SelectedMonth[]>(() => {
    const monthSet = new Set<string>();
    monthSet.add(formatMonthKey(getCurrentMonth()));
    entries.forEach((entry) => monthSet.add(monthKeyFromEntry(entry)));
    return Array.from(monthSet).sort().reverse().map(parseMonthKey);
  }, [entries]);

  const [selectedMonth, setSelectedMonth] =
    useState<SelectedMonth>(getCurrentMonth);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);

  const selectedMonthIndex = availableMonths.findIndex(
    (month) =>
      month.year === selectedMonth.year && month.month === selectedMonth.month,
  );

  const filteredByMonth = useMemo(
    () =>
      entries.filter(
        (entry) => monthKeyFromEntry(entry) === formatMonthKey(selectedMonth),
      ),
    [entries, selectedMonth],
  );

  const availableProviders = useMemo(
    () => Array.from(new Set(entries.map((entry) => entry.provider))).sort(),
    [entries],
  );

  const visibleEntries = useMemo(() => {
    const providerFiltered =
      selectedProvider === null
        ? filteredByMonth
        : filteredByMonth.filter((entry) => entry.provider === selectedProvider);
    return sortEntries(providerFiltered);
  }, [filteredByMonth, selectedProvider]);

  const totalCost = visibleEntries.reduce(
    (sum, entry) => sum + entry.estimatedCost,
    0,
  );

  if (error) {
    return (
      <div className="flex flex-col h-full bg-gray-900 text-white p-8">
        <p className="text-red-400 mb-4">
          Failed to load pricing data: {error}
        </p>
        <button
          onClick={refetch}
          className="self-start px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-sm transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white overflow-hidden">
      <div className="px-6 py-3 border-b border-gray-700/40 flex-shrink-0 space-y-2">
        <div className="flex items-center justify-between gap-4">
          <span className="text-sm font-semibold text-gray-300">Expenses</span>
        </div>
        {!isLoading && visibleEntries.length > 0 && (
          <div className="flex items-baseline justify-center gap-2">
            <span className="text-xs text-gray-500 uppercase tracking-widest">Total</span>
            <span className="text-3xl font-bold text-yellow-300">${totalCost.toFixed(4)}</span>
          </div>
        )}
        <div className="flex items-center justify-between gap-4">
          {availableProviders.length > 1 ? (
            <ProviderFilter
              availableProviders={availableProviders}
              selectedProvider={selectedProvider}
              onSelectProvider={setSelectedProvider}
            />
          ) : (
            <div />
          )}
          <div className="flex items-center gap-1 flex-shrink-0">
            <button
              onClick={() =>
                setSelectedMonth(availableMonths[selectedMonthIndex + 1])
              }
              disabled={selectedMonthIndex >= availableMonths.length - 1}
              className="p-1 rounded text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <span className="text-sm font-medium text-gray-200 w-36 text-center">
              {MONTH_NAMES[selectedMonth.month - 1]} {selectedMonth.year}
            </span>
            <button
              onClick={() =>
                setSelectedMonth(availableMonths[selectedMonthIndex - 1])
              }
              disabled={selectedMonthIndex <= 0}
              className="p-1 rounded text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="p-6">
            <LoadingSpinner message="Loading expenses…" />
          </div>
        ) : visibleEntries.length === 0 ? (
          <div className="p-6">
            <EmptyState
              title="No expenses"
              description="No instances were launched in this period"
            />
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[780px]">
              <thead className="sticky top-0 z-10">
                <tr className="bg-gray-800 text-xs text-gray-500 uppercase tracking-wider border-b border-gray-700">
                  <th className="py-2 px-4 text-left font-medium w-14"></th>
                  <th className="py-2 px-4 text-left font-medium">Instance ID</th>
                  <th className="py-2 px-4 text-left font-medium">Region</th>
                  <th className="py-2 px-4 text-left font-medium">Type</th>
                  <th className="py-2 px-4 text-left font-medium">Launched At</th>
                  <th className="py-2 px-4 text-left font-medium">Terminated At</th>
                  <th className="py-2 px-4 text-left font-medium">Uptime</th>
                  <th className="py-2 px-4 text-left font-medium">Est. Cost</th>
                </tr>
              </thead>
              <tbody>
                {visibleEntries.map((entry) => (
                  <InstanceCostRow key={entry.instanceId} entry={entry} />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
