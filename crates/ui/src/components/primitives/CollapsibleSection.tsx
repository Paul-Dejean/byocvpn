import { ReactNode } from "react";
import { ChevronDown } from "lucide-react";

interface CollapsibleSectionProps {
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  isOpen: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function CollapsibleSection({
  title,
  subtitle,
  icon,
  isOpen,
  onToggle,
  children,
}: CollapsibleSectionProps) {
  return (
    <section className="overflow-hidden rounded-xl border border-bd-moderate bg-bg-bolder">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={isOpen}
        className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-bg-light"
      >
        {icon && <span className="shrink-0 text-fg-medium">{icon}</span>}
        <span className="flex min-w-0 flex-1 flex-col">
          <span className="text-body-sm text-fg-lighter">{title}</span>
          {subtitle && <span className="text-caption text-fg-medium">{subtitle}</span>}
        </span>
        <ChevronDown
          size={16}
          className={`shrink-0 text-fg-medium transition-transform ${isOpen ? "rotate-180" : ""}`}
        />
      </button>
      <div hidden={!isOpen} className="border-t border-bd-faint p-4">
        {children}
      </div>
    </section>
  );
}
