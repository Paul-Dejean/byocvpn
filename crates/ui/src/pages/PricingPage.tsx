import { useMemo, useState } from "react";
import { ChevronLeft, ChevronRight, Wallet } from "lucide-react";
import { useLedger } from "../hooks/useLedger";
import { CalendarMonth, CloudProviderName } from "../types";
import { ProviderFilter } from "../components/pricing/ProviderFilter";
import { InstanceCostRow } from "../components/pricing/InstanceCostRow";
import { LedgerEntryWithCost } from "../types/ledger";
import { Alert } from "../components/primitives/Alert";
import { Button } from "../components/primitives/Button";
import { IconButton } from "../components/primitives/IconButton";
import { Spinner } from "../components/primitives/Spinner";

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

const TABLE_COLUMNS = [
  "",
  "Instance",
  "Region",
  "Type",
  "Launched",
  "Terminated",
  "Uptime",
  "Est. cost",
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
    (sum, entry) => (entry.isPricingUnknown ? sum : sum + entry.estimatedCost),
    0,
  );
  const unknownPricingCount = visibleEntries.filter(
    (entry) => entry.isPricingUnknown,
  ).length;

  return (
    <div className="flex flex-col h-full py-4 pr-4 gap-3">
      <header className="flex flex-col gap-1">
        <h1 className="text-base font-medium text-primary">Expenses</h1>
        <p className="text-xs text-gray-300">
          Estimated cost of every server you launched, by month.
        </p>
      </header>

      {error ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">Failed to load pricing data: {error}</Alert>
          <Button
            variant="secondary"
            size="none"
            onClick={refetch}
            className="px-3 py-1.5 text-sm"
          >
            Retry
          </Button>
        </div>
      ) : (
        <>
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
            <div className="flex items-center gap-1">
              <IconButton
                accent="white"
                size="xs"
                onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex + 1])}
                disabled={calendarMonthIndex >= availableMonths.length - 1}
                aria-label="Previous month"
              >
                <ChevronLeft size={16} />
              </IconButton>
              <span className="text-sm text-primary w-36 text-center">
                {MONTH_NAMES[calendarMonth.month - 1]} {calendarMonth.year}
              </span>
              <IconButton
                accent="white"
                size="xs"
                onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex - 1])}
                disabled={calendarMonthIndex <= 0}
                aria-label="Next month"
              >
                <ChevronRight size={16} />
              </IconButton>
            </div>
          </div>

          <div className="flex-1 min-h-0 rounded-xl bg-gray-750 border border-gray-500/50 flex flex-col overflow-hidden">
            <div className="px-4 py-3 border-b border-gray-500/40 flex items-center justify-between gap-4">
              <span className="text-sm text-primary">
                Servers{" "}
                <span className="text-gray-300">[{visibleEntries.length}]</span>
              </span>
              {!isLoading && visibleEntries.length > 0 && (
                <div className="flex items-baseline gap-2">
                  <span className="text-xs text-gray-300">Total</span>
                  <span className="text-lg text-primary tabular-nums">
                    ${totalCost.toFixed(4)}
                  </span>
                  {unknownPricingCount > 0 && (
                    <span className="text-xs text-gray-400">
                      excludes {unknownPricingCount} with unknown pricing
                    </span>
                  )}
                </div>
              )}
            </div>

            {isLoading ? (
              <div className="flex-1 flex items-center justify-center">
                <Spinner size="w-6 h-6" color="border-gray-400" />
              </div>
            ) : visibleEntries.length === 0 ? (
              <EmptyExpenses />
            ) : (
              <div className="flex-1 min-h-0 overflow-auto">
                <table className="w-full min-w-[780px]">
                  <thead className="sticky top-0 z-10 bg-gray-750">
                    <tr className="text-xs text-gray-300 border-b border-gray-500/40">
                      {TABLE_COLUMNS.map((column, index) => (
                        <th
                          key={index}
                          className={`py-2.5 px-4 text-left font-normal ${index === 0 ? "w-14" : ""}`}
                        >
                          {column}
                        </th>
                      ))}
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
        </>
      )}
    </div>
  );
}

function EmptyExpenses() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-4 text-center">
      <div className="w-16 h-16 rounded-xl border border-dashed border-gray-500 flex items-center justify-center">
        <Wallet size={28} strokeWidth={1.25} className="text-gray-500" />
      </div>
      <div className="flex flex-col gap-1">
        <h2 className="text-sm text-primary">No expenses</h2>
        <p className="text-xs text-gray-300">
          No servers were launched in this period.
        </p>
      </div>
    </div>
  );
}
