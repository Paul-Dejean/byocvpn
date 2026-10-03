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
      className={`flex items-center gap-2 w-full h-9 pl-2 pr-3 rounded-md text-body-sm transition-colors ${
        isActive
          ? "bg-bg-light text-fg-lighter"
          : "text-fg-medium hover:bg-bg-light hover:text-fg-lighter"
      }`}
    >
      <span className="w-4 h-4 flex items-center justify-center">
        {icon}
      </span>
      {label}
    </button>
  );
}
