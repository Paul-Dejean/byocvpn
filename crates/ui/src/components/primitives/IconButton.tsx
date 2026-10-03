import { ButtonHTMLAttributes, ReactNode } from "react";

export type IconButtonAccent = "neutral" | "brand" | "danger" | "warning";
export type IconButtonSize = "xs" | "sm" | "md";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  accent?: IconButtonAccent;
  size?: IconButtonSize;
  children: ReactNode;
}

const ACCENT_CLASSES: Record<IconButtonAccent, string> = {
  neutral: "text-fg-moderate hover:text-fg-lighter",
  brand: "text-fg-moderate hover:text-fg-brand",
  danger: "text-fg-moderate hover:text-fg-danger-moderate",
  warning: "text-fg-warning-moderate hover:text-fg-warning-moderate",
};

const SIZE_CLASSES: Record<IconButtonSize, string> = {
  xs: "p-1",
  sm: "p-1.5",
  md: "p-2",
};

export function IconButton({
  accent = "neutral",
  size = "md",
  className = "",
  children,
  ...buttonProps
}: IconButtonProps) {
  return (
    <button
      className={`${SIZE_CLASSES[size]} rounded-lg transition-colors hover:bg-bg-light disabled:opacity-40 disabled:cursor-not-allowed ${ACCENT_CLASSES[accent]} ${className}`}
      {...buttonProps}
    >
      {children}
    </button>
  );
}
