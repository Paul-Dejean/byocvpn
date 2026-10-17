import { useState } from "react";
import { ChevronLeft, ChevronRight, Wallet } from "lucide-react";
import { useLedger } from "../hooks/useLedger";
import { useIsMobileLayout } from "../hooks/useIsMobileLayout";
import { CalendarMonth, CloudProviderName } from "../types";
import { ProviderFilter } from "../components/pricing/ProviderFilter";
import { InstanceCostRow } from "../components/pricing/InstanceCostRow";
import { InstanceCostCard } from "../components/pricing/InstanceCostCard";
import { LedgerEntryWithCost } from "../types/ledger";
import { Banner } from "../components/primitives/Banner";
import { Button } from "../components/primitives/Button";
import { IconButton } from "../components/primitives/IconButton";
import { Spinner } from "../components/primitives/Spinner";

const MONTH_NAMES = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

const TABLE_COLUMNS: { id: string; label: string; width: string }[] = [
  { id: "status", label: "", width: "44px" },
  { id: "instance", label: "Instance", width: "auto" },
  { id: "region", label: "Region", width: "112px" },
  { id: "type", label: "Type", width: "150px" },
  { id: "launched", label: "Launched", width: "104px" },
  { id: "terminated", label: "Terminated", width: "104px" },
  { id: "uptime", label: "Uptime", width: "64px" },
  { id: "cost", label: "Cost", width: "104px" },
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
  return [...entries].toSorted((entryA, entryB) => {
    if (entryA.terminatedAt === null && entryB.terminatedAt !== null) return -1;
    if (entryA.terminatedAt !== null && entryB.terminatedAt === null) return 1;
    const dateA = entryA.terminatedAt ?? entryA.launchedAt;
    const dateB = entryB.terminatedAt ?? entryB.launchedAt;
    return new Date(dateB).getTime() - new Date(dateA).getTime();
  });
}

export function PricingPage() {
  const { entries, isLoading, error, refetch } = useLedger();
  const isMobileLayout = useIsMobileLayout();

  const monthSet = new Set<string>();
  monthSet.add(formatMonthKey(getCurrentMonth()));
  entries.forEach((entry) => monthSet.add(monthKeyFromEntry(entry)));
  const availableMonths: CalendarMonth[] = Array.from(monthSet)
    .toSorted()
    .toReversed()
    .map(parseMonthKey);

  const [calendarMonth, setCalendarMonth] = useState<CalendarMonth>(getCurrentMonth);
  const [selectedProvider, setSelectedProvider] = useState<CloudProviderName | null>(null);

  const calendarMonthIndex = availableMonths.findIndex(
    (month) => month.year === calendarMonth.year && month.month === calendarMonth.month,
  );

  const filteredByMonth = entries.filter(
    (entry) => monthKeyFromEntry(entry) === formatMonthKey(calendarMonth),
  );

  const availableProviders = Array.from(new Set(entries.map((entry) => entry.provider))).toSorted();

  const providerFiltered =
    selectedProvider === null
      ? filteredByMonth
      : filteredByMonth.filter((entry) => entry.provider === selectedProvider);
  const visibleEntries = sortEntries(providerFiltered);

  const totalCost = visibleEntries.reduce(
    (sum, entry) => (entry.isPricingUnknown ? sum : sum + entry.estimatedCost),
    0,
  );
  const unknownPricingCount = visibleEntries.filter((entry) => entry.isPricingUnknown).length;

  return (
    <div className="flex h-full flex-col gap-3">
      <header className="flex flex-col gap-1">
        <h1 className="text-body-sm font-medium text-fg-lighter">Expenses</h1>
        <p className="text-caption text-fg-medium">
          Estimated cost of every server you launched, by month.
        </p>
      </header>

      {error ? (
        <div className="flex flex-col items-start gap-3">
          <Banner variant="danger">Failed to load pricing data: {error}</Banner>
          <Button variant="secondary" size="lg" onClick={refetch}>
            Retry
          </Button>
        </div>
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
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
                accent="neutral"
                size="xs"
                onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex + 1])}
                disabled={calendarMonthIndex >= availableMonths.length - 1}
                aria-label="Previous month"
              >
                <ChevronLeft size={16} />
              </IconButton>
              <span className="w-36 text-center text-body-sm text-fg-lighter">
                {MONTH_NAMES[calendarMonth.month - 1]} {calendarMonth.year}
              </span>
              <IconButton
                accent="neutral"
                size="xs"
                onClick={() => setCalendarMonth(availableMonths[calendarMonthIndex - 1])}
                disabled={calendarMonthIndex <= 0}
                aria-label="Next month"
              >
                <ChevronRight size={16} />
              </IconButton>
            </div>
          </div>

          <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-bd-moderate bg-bg-bolder">
            <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 border-b border-bd-faint px-4 py-3">
              <span className="text-body-sm text-fg-lighter">
                Servers <span className="text-fg-medium">[{visibleEntries.length}]</span>
              </span>
              {!isLoading && visibleEntries.length > 0 && (
                <div className="flex flex-wrap items-baseline gap-x-2">
                  <span className="text-caption text-fg-medium">Total</span>
                  <span className="text-feature text-fg-lighter tabular-nums">
                    ${totalCost.toFixed(4)}
                  </span>
                  {unknownPricingCount > 0 && (
                    <span className="text-caption text-fg-moderate">
                      excludes {unknownPricingCount} with unknown pricing
                    </span>
                  )}
                </div>
              )}
            </div>

            {isLoading ? (
              <div className="flex flex-1 items-center justify-center">
                <Spinner size="w-6 h-6" color="border-bd-strong" />
              </div>
            ) : visibleEntries.length === 0 ? (
              <EmptyExpenses />
            ) : isMobileLayout ? (
              <div className="min-h-0 flex-1 overflow-y-auto">
                {visibleEntries.map((entry) => (
                  <InstanceCostCard key={entry.instanceId} entry={entry} />
                ))}
              </div>
            ) : (
              <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
                <table className="w-full table-fixed">
                  <colgroup>
                    {TABLE_COLUMNS.map((column) => (
                      <col key={column.id} style={{ width: column.width }} />
                    ))}
                  </colgroup>
                  <thead className="sticky top-0 z-10 bg-bg-bolder">
                    <tr className="border-b border-bd-faint text-caption text-fg-medium">
                      {TABLE_COLUMNS.map((column) => (
                        <th
                          key={column.id}
                          className="px-3 py-2.5 text-left font-normal whitespace-nowrap"
                        >
                          {column.label}
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
    <div className="flex flex-1 flex-col items-center justify-center gap-4 text-center">
      <div className="flex size-16 items-center justify-center rounded-xl border border-dashed border-bd-moderate">
        <Wallet size={28} strokeWidth={1.25} className="text-fg-moderate" />
      </div>
      <div className="flex flex-col gap-1">
        <h2 className="text-body-sm text-fg-lighter">No expenses</h2>
        <p className="text-caption text-fg-medium">No servers were launched in this period.</p>
      </div>
    </div>
  );
}
