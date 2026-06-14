import { ReactNode } from "react";
import { IconButton } from "./IconButton";

interface DrawerProps {
  isOpen: boolean;
  onClose: () => void;
  title: ReactNode;
  subtitle?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}

export function Drawer({ isOpen, onClose, title, subtitle, footer, children }: DrawerProps) {
  return (
    <>
      <div
        className={`fixed inset-0 z-40 bg-overlay transition-opacity duration-300 ${
          isOpen
            ? "opacity-50 pointer-events-auto"
            : "opacity-0 pointer-events-none"
        }`}
        onClick={onClose}
      />

      <div
        className={`fixed top-0 right-0 h-full w-96 bg-gray-800 z-50 flex flex-col shadow-2xl border-l border-gray-700/50 transition-transform duration-300 ease-in-out ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <div className="flex items-center justify-between p-6 border-b border-gray-700/50 flex-shrink-0">
          <div>
            <h2 className="text-lg font-semibold text-primary">{title}</h2>
            {subtitle && (
              <p className="text-sm text-gray-400 mt-1">{subtitle}</p>
            )}
          </div>
          <IconButton accent="white" onClick={onClose}>
            <svg
              className="w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </IconButton>
        </div>

        <div className="flex-1 overflow-y-auto p-6">{children}</div>

        {footer && (
          <div className="p-6 border-t border-gray-700/50 flex-shrink-0">
            {footer}
          </div>
        )}
      </div>
    </>
  );
}
