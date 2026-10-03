import { ReactNode } from "react";

interface FilterChipProps {
  selected: boolean;
  onClick: () => void;
  children: ReactNode;
}

export function FilterChip({ selected, onClick, children }: FilterChipProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs border transition-colors ${
        selected
          ? "bg-gray-700 border-blue-500 text-primary"
          : "bg-gray-750 border-gray-500/60 text-gray-200 hover:bg-gray-700 hover:text-primary"
      }`}
    >
      {children}
    </button>
  );
}
