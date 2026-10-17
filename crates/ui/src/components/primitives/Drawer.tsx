import { ReactNode } from "react";
import { X } from "lucide-react";
import { useIsMobileLayout } from "../../hooks/useIsMobileLayout";
import { IconButton } from "./IconButton";
import { MobileSheet } from "./MobileSheet";

interface DrawerProps {
  isOpen: boolean;
  onClose: () => void;
  title: ReactNode;
  subtitle?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}

export function Drawer({ isOpen, onClose, title, subtitle, footer, children }: DrawerProps) {
  const isMobileLayout = useIsMobileLayout();

  if (isMobileLayout) {
    return (
      <MobileSheet isOpen={isOpen} onClose={onClose}>
        <DrawerContent title={title} subtitle={subtitle} footer={footer} onClose={onClose}>
          {children}
        </DrawerContent>
      </MobileSheet>
    );
  }

  return (
    <>
      <div
        className={`absolute inset-0 z-40 bg-overlay transition-opacity duration-300 ${
          isOpen ? "pointer-events-auto opacity-60" : "pointer-events-none opacity-0"
        }`}
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        className={`absolute top-0 right-0 z-50 flex h-full w-100 flex-col border-l border-bd-moderate bg-bg-strong shadow-2xl transition-transform duration-300 ease-in-out ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <DrawerContent title={title} subtitle={subtitle} footer={footer} onClose={onClose}>
          {children}
        </DrawerContent>
      </div>
    </>
  );
}

interface DrawerContentProps {
  title: ReactNode;
  subtitle?: ReactNode;
  footer?: ReactNode;
  onClose: () => void;
  children: ReactNode;
}

function DrawerContent({ title, subtitle, footer, onClose, children }: DrawerContentProps) {
  return (
    <>
      <div className="flex shrink-0 items-start justify-between gap-4 border-b border-bd-faint px-4 py-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h2 className="text-body-sm font-medium text-fg-lighter">{title}</h2>
          {subtitle && <p className="text-caption text-fg-medium">{subtitle}</p>}
        </div>
        <IconButton size="sm" onClick={onClose} aria-label="Close">
          <X size={16} />
        </IconButton>
      </div>

      <div className="flex-1 overflow-y-auto p-4">{children}</div>

      {footer && <div className="shrink-0 border-t border-bd-faint p-4">{footer}</div>}
    </>
  );
}
