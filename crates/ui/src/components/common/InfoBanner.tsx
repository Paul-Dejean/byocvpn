import { Info, X } from "lucide-react";
import { ReactNode } from "react";

interface InfoBannerProps {
  children: ReactNode;
  onDismiss: () => void;
}

export function InfoBanner({ children, onDismiss }: InfoBannerProps) {
  return (
    <div className="flex items-start gap-3 rounded-lg bg-blue-900/70 border border-blue-700/50 px-3 py-2.5 text-sm text-primary">
      <Info size={18} className="flex-shrink-0 mt-0.5 text-blue-300" />
      <p className="flex-1 leading-snug">{children}</p>
      <button
        type="button"
        onClick={onDismiss}
        aria-label="Dismiss"
        className="flex-shrink-0 text-gray-300 hover:text-primary transition-colors"
      >
        <X size={16} />
      </button>
    </div>
  );
}
