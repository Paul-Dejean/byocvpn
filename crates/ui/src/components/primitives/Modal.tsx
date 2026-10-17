import { ReactNode, useEffect } from "react";
import { useIsMobileLayout } from "../../hooks/useIsMobileLayout";
import { MobileSheet } from "./MobileSheet";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
  className?: string;
}

export function Modal({ isOpen, onClose, children, className = "" }: ModalProps) {
  const isMobileLayout = useIsMobileLayout();

  useEffect(() => {
    if (!isOpen) {
      return undefined;
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (isMobileLayout) {
    return (
      <MobileSheet isOpen={isOpen} onClose={onClose} isFullHeight>
        {children}
      </MobileSheet>
    );
  }

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="absolute inset-0 z-40 flex items-center justify-center bg-overlay/60"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        className={`flex max-h-160 w-[626px] flex-col overflow-hidden rounded-xl border border-bd-moderate bg-bg-bolder shadow-2xl ${className}`}
      >
        {children}
      </div>
    </div>
  );
}
