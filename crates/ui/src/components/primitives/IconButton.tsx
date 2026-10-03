import { ButtonHTMLAttributes, ReactNode } from "react";

export type IconButtonAccent = "white" | "blue" | "red" | "amber";
export type IconButtonSize = "xs" | "sm" | "md";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  accent?: IconButtonAccent;
  size?: IconButtonSize;
  children: ReactNode;
}

const ACCENT_CLASSES: Record<IconButtonAccent, string> = {
  white: "text-fg-moderate hover:text-fg-lighter",
  blue: "text-fg-moderate hover:text-fg-brand",
  red: "text-fg-moderate hover:text-fg-danger-moderate",
  amber: "text-fg-warning-moderate hover:text-fg-warning-moderate",
};

const SIZE_CLASSES: Record<IconButtonSize, string> = {
  xs: "p-1",
  sm: "p-1.5",
  md: "p-2",
};

export function IconButton({
  accent = "white",
  size = "md",
  className = "",
  children,
  ...buttonProps
}: IconButtonProps) {
  return (
    <button
      className={`${SIZE_CLASSES[size]} rounded-lg transition-colors hover:bg-bg-lighter disabled:opacity-40 disabled:cursor-not-allowed ${ACCENT_CLASSES[accent]} ${className}`}
      {...buttonProps}
    >
      {children}
    </button>
  );
}
