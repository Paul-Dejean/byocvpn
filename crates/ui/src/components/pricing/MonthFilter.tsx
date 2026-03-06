export interface SelectedMonth {
  year: number;
  /** 1-indexed month number (1 = January … 12 = December) */
  month: number;
}

const MONTH_NAMES = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

/**
 * Props for the MonthFilter component
 */
interface MonthFilterProps {
  /** All months that have data (plus the current month) */
  availableMonths: SelectedMonth[];
  /** The currently selected month */
  selectedMonth: SelectedMonth;
  /** Callback when the user picks a different month */
  onSelectMonth: (month: SelectedMonth) => void;
}

/**
 * Pill-style month selector shown at the top of the Pricing page.
 */
export function MonthFilter({
  availableMonths,
  selectedMonth,
  onSelectMonth,
}: MonthFilterProps) {
  return (
    <div className="flex items-center gap-2 flex-wrap">
      {availableMonths.map((month) => {
        const isSelected =
          month.year === selectedMonth.year &&
          month.month === selectedMonth.month;
        return (
          <button
            key={`${month.year}-${month.month}`}
            onClick={() => onSelectMonth(month)}
            className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
              isSelected
                ? "bg-blue-500 text-white"
                : "bg-gray-700 text-gray-300 hover:bg-gray-600"
            }`}
          >
            {MONTH_NAMES[month.month - 1]} {month.year}
          </button>
        );
      })}
    </div>
  );
}
