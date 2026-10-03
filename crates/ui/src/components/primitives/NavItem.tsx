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
      className={`flex items-center gap-2 w-full h-9 pl-2 pr-3 rounded-md text-sm transition-colors ${
        isActive
          ? "bg-white/5 text-primary"
          : "text-gray-200 hover:bg-white/5 hover:text-primary"
      }`}
    >
      <span className="w-4 h-4 flex items-center justify-center text-gray-200">
        {icon}
      </span>
      {label}
    </button>
  );
}
