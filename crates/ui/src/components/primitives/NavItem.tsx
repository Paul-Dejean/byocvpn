import { ReactNode, forwardRef } from "react";

interface NavItemProps {
  icon: ReactNode;
  label: string;
  isActive: boolean;
  onClick: () => void;
}

export const NavItem = forwardRef<HTMLButtonElement, NavItemProps>(
  function NavItem({ icon, label, isActive, onClick }, ref) {
    return (
      <button
        ref={ref}
        onClick={onClick}
        title={label}
        className={`relative flex flex-col items-center justify-center w-10 h-10 rounded-lg transition-colors duration-300 ${
          isActive
            ? "text-white"
            : "text-gray-400 hover:bg-gray-700 hover:text-primary"
        }`}
      >
        {icon}
      </button>
    );
  },
);
