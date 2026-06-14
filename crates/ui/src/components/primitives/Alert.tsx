import { ReactNode } from "react";

export type AlertVariant = "success" | "error" | "warning";

interface AlertProps {
  variant: AlertVariant;
  children?: ReactNode;
  title?: ReactNode;
  icon?: ReactNode;
  className?: string;
}

const VARIANT_CLASSES: Record<AlertVariant, string> = {
  success: "bg-success-900 border-success-700 text-success-300",
  error: "bg-danger-900 border-danger-700 text-danger-300",
  warning: "bg-warning-900 border-warning-700 text-warning-300",
};

export function Alert({ variant, children, title, icon, className = "" }: AlertProps) {
  return (
    <div
      className={`flex items-start gap-2.5 p-3 rounded-lg border text-sm ${VARIANT_CLASSES[variant]} ${className}`}
    >
      {icon && <span className="flex-shrink-0 mt-0.5">{icon}</span>}
      <div className="min-w-0">
        {title && <p className="font-medium">{title}</p>}
        {children && (
          <div className={title ? "text-xs opacity-80 mt-0.5 break-words" : ""}>
            {children}
          </div>
        )}
      </div>
    </div>
  );
}
