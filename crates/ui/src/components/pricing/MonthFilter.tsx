export interface SelectedMonth {
  year: number;

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

interface MonthFilterProps {

  availableMonths: SelectedMonth[];

  selectedMonth: SelectedMonth;

  onSelectMonth: (month: SelectedMonth) => void;
}

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
            className={`px-3 py-1 rounded-lg text-sm font-medium transition-colors ${
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
