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
  info: "bg-bg-brand-faint text-fg-lighter",
  neutral: "bg-bg-medium text-fg-lighter",
  success: "bg-bg-success-faint text-fg-lighter",
  warning: "bg-bg-warning-faint text-fg-lighter",
  danger: "bg-bg-danger-faint text-fg-lighter",
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
      className={`flex items-start gap-3 rounded-xl px-4 py-3 text-body-sm ${VARIANT_CLASSES[variant]} ${className}`}
    >
      <span className={`shrink-0 ${ICON_CLASSES[variant]}`}>
        {icon ?? <DefaultIcon variant={variant} />}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        {title && <p className="font-medium">{title}</p>}
        {children && (
          <div className={`wrap-break-word ${title ? "text-caption text-fg-medium" : ""}`}>
            {children}
          </div>
        )}
      </div>
      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          aria-label="Dismiss"
          className="shrink-0 text-fg-medium transition-colors hover:text-fg-lighter"
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
      return <Check size={20} />;
    case "warning":
    case "danger":
      return <AlertTriangle size={20} />;
    case "info":
    case "neutral":
      return <Info size={20} />;
    default:
      return null;
  }
}
