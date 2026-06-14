import { ButtonHTMLAttributes, ReactNode } from "react";

export type IconButtonAccent = "white" | "blue" | "red" | "amber";
export type IconButtonSize = "xs" | "sm" | "md";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  accent?: IconButtonAccent;
  size?: IconButtonSize;
  children: ReactNode;
}

const ACCENT_CLASSES: Record<IconButtonAccent, string> = {
  white: "text-gray-400 hover:text-primary",
  blue: "text-gray-400 hover:text-blue-400",
  red: "text-gray-400 hover:text-danger-400",
  amber: "text-warning-400 hover:text-warning-300",
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
      className={`${SIZE_CLASSES[size]} rounded-lg transition-colors hover:bg-gray-600 disabled:opacity-40 disabled:cursor-not-allowed ${ACCENT_CLASSES[accent]} ${className}`}
      {...buttonProps}
    >
      {children}
    </button>
  );
}
