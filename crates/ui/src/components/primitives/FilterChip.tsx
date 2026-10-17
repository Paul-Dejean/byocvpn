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
      className={`flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-caption transition-colors ${
        selected
          ? "border-bd-brand bg-bg-medium text-fg-lighter"
          : "border-bd-moderate bg-bg-bolder text-fg-medium hover:bg-bg-medium hover:text-fg-lighter"
      }`}
    >
      {children}
    </button>
  );
}
