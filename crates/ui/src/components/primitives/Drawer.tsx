import { ReactNode } from "react";
import { X } from "lucide-react";
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
        className={`absolute inset-0 z-40 bg-overlay transition-opacity duration-300 ${
          isOpen
            ? "opacity-60 pointer-events-auto"
            : "opacity-0 pointer-events-none"
        }`}
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        className={`absolute top-0 right-0 h-full w-[400px] bg-bg-strong z-50 flex flex-col border-l border-bd-moderate shadow-2xl transition-transform duration-300 ease-in-out ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <div className="flex items-start justify-between gap-4 p-5 border-b border-bd-faint flex-shrink-0">
          <div className="flex flex-col gap-0.5 min-w-0">
            <h2 className="text-body font-medium text-fg-lighter">{title}</h2>
            {subtitle && <p className="text-caption text-fg-medium">{subtitle}</p>}
          </div>
          <IconButton accent="white" size="sm" onClick={onClose} aria-label="Close">
            <X size={18} />
          </IconButton>
        </div>

        <div className="flex-1 overflow-y-auto p-5">{children}</div>

        {footer && (
          <div className="p-5 border-t border-bd-faint flex-shrink-0">
            {footer}
          </div>
        )}
      </div>
    </>
  );
}
