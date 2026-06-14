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
      onClick={onClick}
      title={label}
      className={`flex flex-col items-center justify-center w-10 h-10 rounded-lg transition-colors ${
        isActive
          ? "bg-blue-600 text-white"
          : "text-gray-400 hover:bg-gray-700 hover:text-primary"
      }`}
    >
      {icon}
    </button>
  );
}
