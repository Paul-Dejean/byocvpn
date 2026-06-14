import { ReactNode } from "react";
import { Spinner } from "./Spinner";

export type BadgeVariant = "success" | "warning" | "danger" | "info" | "neutral";
export type BadgeShape = "pill" | "square";

interface BadgeProps {
  variant: BadgeVariant;
  shape?: BadgeShape;
  spinner?: boolean;
  children: ReactNode;
}

const VARIANT_CLASSES: Record<BadgeVariant, { fill: string; border: string }> = {
  success: { fill: "bg-success-900/50 text-success-300", border: "border-success-700/50" },
  warning: { fill: "bg-warning-900/50 text-warning-300", border: "border-warning-700/50" },
  danger: { fill: "bg-danger-900/50 text-danger-300", border: "border-danger-700/50" },
  info: { fill: "bg-blue-900/50 text-blue-300", border: "border-blue-700/50" },
  neutral: { fill: "bg-gray-700/50 text-gray-400", border: "border-gray-600/50" },
};

export function Badge({ variant, shape = "square", spinner = false, children }: BadgeProps) {
  const { fill, border } = VARIANT_CLASSES[variant];
  const shapeClasses =
    shape === "pill"
      ? `px-2 py-0.5 rounded-full border ${border}`
      : "px-2 py-1 rounded";

  return (
    <span
      className={`inline-flex items-center gap-1 text-xs font-medium ${fill} ${shapeClasses}`}
    >
      {spinner && <Spinner />}
      {children}
    </span>
  );
}
