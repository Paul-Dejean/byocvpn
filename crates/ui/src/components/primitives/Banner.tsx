import { ReactNode } from "react";
import { AlertTriangle, Check, Info, X } from "lucide-react";

export type BannerVariant = "info" | "neutral" | "success" | "warning" | "danger";

interface BannerProps {
  variant?: BannerVariant;
  title?: ReactNode;
  icon?: ReactNode;
  onDismiss?: () => void;
  children?: ReactNode;
  className?: string;
}

const VARIANT_CLASSES: Record<BannerVariant, string> = {
  info: "bg-bg-brand-faint border-bd-brand/50 text-fg-lighter",
  neutral: "bg-bg-medium border-bd-moderate text-fg-lighter",
  success: "bg-bg-success-faint border-bd-success text-fg-lighter",
  warning: "bg-bg-warning-faint border-bd-warning text-fg-lighter",
  danger: "bg-bg-danger-faint border-bd-danger text-fg-lighter",
};

const ICON_CLASSES: Record<BannerVariant, string> = {
  info: "text-fg-brand",
  neutral: "text-fg-medium",
  success: "text-fg-success-moderate",
  warning: "text-fg-warning-moderate",
  danger: "text-fg-danger-moderate",
};

export function Banner({
  variant = "neutral",
  title,
  icon,
  onDismiss,
  children,
  className = "",
}: BannerProps) {
  return (
    <div
      role={variant === "danger" ? "alert" : "status"}
      className={`flex items-start gap-3 rounded-lg border px-3 py-2.5 text-body-sm ${VARIANT_CLASSES[variant]} ${className}`}
    >
      <span className={`flex-shrink-0 mt-0.5 ${ICON_CLASSES[variant]}`}>
        {icon ?? <DefaultIcon variant={variant} />}
      </span>
      <div className="flex-1 min-w-0 flex flex-col gap-0.5">
        {title && <p className="font-medium">{title}</p>}
        {children && (
          <div className={`break-words leading-snug ${title ? "text-caption text-fg-medium" : ""}`}>
            {children}
          </div>
        )}
      </div>
      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          aria-label="Dismiss"
          className="flex-shrink-0 text-fg-medium hover:text-fg-lighter transition-colors"
        >
          <X size={16} />
        </button>
      )}
    </div>
  );
}

function DefaultIcon({ variant }: { variant: BannerVariant }) {
  switch (variant) {
    case "success":
      return <Check size={16} />;
    case "warning":
    case "danger":
      return <AlertTriangle size={16} />;
    case "info":
    case "neutral":
      return <Info size={16} />;
  }
}
