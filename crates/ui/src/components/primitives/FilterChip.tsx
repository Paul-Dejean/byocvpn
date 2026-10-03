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
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-caption border transition-colors ${
        selected
          ? "bg-bg-medium border-bd-brand text-fg-lighter"
          : "bg-bg-bolder border-bd-moderate text-fg-medium hover:bg-bg-medium hover:text-fg-lighter"
      }`}
    >
      {children}
    </button>
  );
}
