import { ReactNode } from "react";

interface FilterChipProps {
  selected: boolean;
  onClick: () => void;
  children: ReactNode;
}

export function FilterChip({ selected, onClick, children }: FilterChipProps) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 px-3 py-1 rounded-lg text-sm font-medium transition-colors ${
        selected
          ? "bg-blue-500 text-white"
          : "bg-gray-700 text-gray-300 hover:bg-gray-600"
      }`}
    >
      {children}
    </button>
  );
}
