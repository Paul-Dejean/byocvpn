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
      className={`flex items-center gap-3 w-full px-3 py-2 rounded-lg text-sm transition-colors ${
        isActive
          ? "bg-gray-700 text-primary"
          : "text-gray-200 hover:bg-gray-750 hover:text-primary"
      }`}
    >
      <span className="w-4 h-4 flex items-center justify-center text-gray-200">
        {icon}
      </span>
      {label}
    </button>
  );
}
