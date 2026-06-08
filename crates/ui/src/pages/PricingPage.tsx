import { useMemo, useState } from "react";
import { useLedger } from "../hooks/useLedger";
import { CalendarMonth, CloudProviderName } from "../types";
import { ProviderFilter } from "../components/pricing/ProviderFilter";
import { InstanceCostRow } from "../components/pricing/InstanceCostRow";
import { LoadingScreen } from "../components/common/LoadingScreen";
import { EmptyState } from "../components/common/EmptyState";
import { LedgerEntryWithCost } from "../types/ledger";

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

function getCurrentMonth(): CalendarMonth {
  const now = new Date();
  return { year: now.getFullYear(), month: now.getMonth() + 1 };
}

function monthKeyFromEntry(entry: LedgerEntryWithCost): string {
  const date = new Date(entry.launchedAt);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
}

function parseMonthKey(key: string): CalendarMonth {
  const [year, month] = key.split("-").map(Number);
  return { year, month };
}

function formatMonthKey(month: CalendarMonth): string {
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

  const availableMonths = useMemo<CalendarMonth[]>(() => {
    const monthSet = new Set<string>();
    monthSet.add(formatMonthKey(getCurrentMonth()));
    entries.forEach((entry) => monthSet.add(monthKeyFromEntry(entry)));
    return Array.from(monthSet).sort().reverse().map(parseMonthKey);
  }, [entries]);

  const [calendarMonth, setCalendarMonth] =
    useState<CalendarMonth>(getCurrentMonth);
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName | null>(null);

  const calendarMonthIndex = availableMonths.findIndex(
    (month) =>
      month.year === calendarMonth.year && month.month === calendarMonth.month,
  );

  const filteredByMonth = useMemo(
    () =>
      entries.filter(
        (entry) => monthKeyFromEntry(entry) === formatMonthKey(calendarMonth),
      ),
    [entries, calendarMonth],
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
          className="btn-primary self-start px-4 py-2 text-sm"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white overflow-hidden">
      <div className="px-6 flex-shrink-0">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-500 pt-6 pb-2 border-b border-gray-700/50">
          Expenses
        </h2>
        <div className="flex items-center justify-between gap-4 py-4">
          {availableProviders.length > 1 ? (
            <ProviderFilter
              availableProviders={availableProviders}
              selectedProvider={selectedProvider}
              onSelectProvider={setSelectedProvider}
            />
          ) : (
            <div />
          )}
          {!isLoading && visibleEntries.length > 0 && (
            <div className="flex items-baseline gap-2">
              <span className="text-xs text-gray-500 uppercase tracking-widest">Total</span>
              <span className="text-2xl font-bold text-yellow-300">${totalCost.toFixed(4)}</span>
            </div>
          )}
          <div className="flex items-center gap-1 flex-shrink-0">
            <button
              onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex + 1])}
              disabled={calendarMonthIndex >= availableMonths.length - 1}
              className="p-1 rounded-lg text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <span className="text-sm font-medium text-gray-200 w-36 text-center">
              {MONTH_NAMES[calendarMonth.month - 1]} {calendarMonth.year}
            </span>
            <button
              onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex - 1])}
              disabled={calendarMonthIndex <= 0}
              className="p-1 rounded-lg text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
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
            <LoadingScreen message="Loading expenses…" />
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
              <thead className="sticky top-0 z-10 bg-gray-800/60">
                <tr className="text-xs font-semibold uppercase tracking-wider text-gray-300 border-b border-gray-600/60">
                  <th className="py-3 px-4 text-left w-14"></th>
                  <th className="py-3 px-4 text-left">Instance ID</th>
                  <th className="py-3 px-4 text-left">Region</th>
                  <th className="py-3 px-4 text-left">Type</th>
                  <th className="py-3 px-4 text-left">Launched At</th>
                  <th className="py-3 px-4 text-left">Terminated At</th>
                  <th className="py-3 px-4 text-left">Uptime</th>
                  <th className="py-3 px-4 text-left">Est. Cost</th>
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
