import { useMemo, useState } from "react";
import { useLedger } from "../hooks/useLedger";
import { MonthFilter, SelectedMonth } from "../components/pricing/MonthFilter";
import { PricingAccordion } from "../components/pricing/PricingAccordion";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { EmptyState } from "../components/common/EmptyState";
import { LedgerEntryWithCost } from "../types/ledger";

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

  const filteredEntries = useMemo(
    () =>
      entries.filter((entry) => {
        const key = monthKeyFromEntry(entry);
        return key === formatMonthKey(selectedMonth);
      }),
    [entries, selectedMonth],
  );

  const groupedByProvider = useMemo(() => {
    const map = new Map<string, LedgerEntryWithCost[]>();
    filteredEntries.forEach((entry) => {
      const existing = map.get(entry.provider) ?? [];
      map.set(entry.provider, [...existing, entry]);
    });
    return Array.from(map.entries()).sort(
      ([, a], [, b]) =>
        b.reduce((sum, entry) => sum + entry.estimatedCost, 0) -
        a.reduce((sum, entry) => sum + entry.estimatedCost, 0),
    );
  }, [filteredEntries]);

  const totalCost = filteredEntries.reduce(
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
      {}
      <div className="px-6 py-3 border-b border-gray-700/40 flex-shrink-0">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <span className="text-sm font-semibold text-gray-300">Expenses</span>

          {!isLoading && filteredEntries.length > 0 && (
            <div className="text-right">
              <p className="text-xs text-gray-500 uppercase tracking-wider">
                Total this period
              </p>
              <p className="text-2xl font-bold text-yellow-300">
                ${totalCost.toFixed(4)}
              </p>
            </div>
          )}
        </div>

        {}
        <div className="mt-4">
          <MonthFilter
            availableMonths={availableMonths}
            selectedMonth={selectedMonth}
            onSelectMonth={setSelectedMonth}
          />
        </div>
      </div>

      {}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {isLoading ? (
          <LoadingSpinner message="Loading expenses…" />
        ) : groupedByProvider.length === 0 ? (
          <EmptyState
            title="No expenses"
            description="No instances were launched in this period"
          />
        ) : (
          groupedByProvider.map(([provider, providerEntries]) => (
            <PricingAccordion
              key={provider}
              provider={provider}
              entries={providerEntries}
            />
          ))
        )}
      </div>
    </div>
  );
}
