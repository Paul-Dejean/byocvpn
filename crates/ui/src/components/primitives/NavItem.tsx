import { ReactNode } from "react";

interface NavItemProps {
  icon: ReactNode;
  label: string;
  isActive: boolean;
  onClick: () => void;
}

export function NavItem({ icon, label, isActive, onClick }: NavItemProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex h-9 w-full items-center gap-2 rounded-md pr-3 pl-2 text-body-sm transition-colors ${
        isActive
          ? "bg-bg-light text-fg-lighter"
          : "text-fg-medium hover:bg-bg-light hover:text-fg-lighter"
      }`}
    >
      <span className="flex size-4 items-center justify-center">{icon}</span>
      {label}
    </button>
  );
}
