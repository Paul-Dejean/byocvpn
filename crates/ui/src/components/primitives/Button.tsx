import { ButtonHTMLAttributes, ReactNode } from "react";
import { Spinner } from "./Spinner";

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "danger"
  | "ghostDanger"
  | "success"
  | "ghost";
export type ButtonSize = "sm" | "md" | "lg" | "none";
export type ButtonDisabledStyle = "grey" | "dim";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  disabledStyle?: ButtonDisabledStyle;
  icon?: ReactNode;
  children: ReactNode;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary: "btn-primary",
  secondary: "btn-secondary",
  danger: "btn-danger",
  ghostDanger: "btn-ghost-danger",
  success: "bg-success-600 hover:bg-success-700 text-white rounded-lg font-medium transition-colors",
  ghost: "bg-transparent border border-gray-600 hover:border-gray-400 text-gray-400 hover:text-primary rounded-lg transition-colors",
};

const SIZE_CLASSES: Record<ButtonSize, string> = {
  sm: "px-3 py-1.5 text-sm",
  md: "px-4 py-2",
  lg: "px-6 py-4 text-lg",
  none: "",
};

const DISABLED_CLASSES: Record<ButtonDisabledStyle, string> = {
  grey: "disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-600",
  dim: "disabled:opacity-50 disabled:cursor-not-allowed",
};

const SPINNER_COLOR: Record<ButtonVariant, string> = {
  primary: "border-white",
  danger: "border-white",
  success: "border-white",
  secondary: "border-current",
  ghostDanger: "border-current",
  ghost: "border-current",
};

export function Button({
  variant = "primary",
  size = "md",
  loading = false,
  disabledStyle = "grey",
  icon,
  disabled,
  className = "",
  children,
  ...buttonProps
}: ButtonProps) {
  return (
    <button
      disabled={disabled || loading}
      className={`inline-flex items-center justify-center gap-2 ${VARIANT_CLASSES[variant]} ${SIZE_CLASSES[size]} ${DISABLED_CLASSES[disabledStyle]} ${className}`}
      {...buttonProps}
    >
      {loading ? <Spinner color={SPINNER_COLOR[variant]} /> : icon}
      {children}
    </button>
  );
}
