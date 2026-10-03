import { ButtonHTMLAttributes, ReactNode } from "react";
import { Spinner } from "./Spinner";

export type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
export type ButtonSize = "md" | "lg";
export type ButtonDisabledStyle = "grey" | "dim";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  disabledStyle?: ButtonDisabledStyle;
  icon?: ReactNode;
  trailingIcon?: ReactNode;
  children: ReactNode;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary: "bg-bg-brand-bolder hover:bg-bg-brand-bold text-fg-on-brand",
  secondary:
    "bg-bg-light hover:bg-bg-lighter border border-bd-moderate text-fg-lighter",
  danger: "bg-bg-danger-moderate hover:bg-bg-danger-bold text-fg-on-brand",
  ghost: "bg-transparent hover:bg-bg-light text-fg-lighter",
};

const SIZE_CLASSES: Record<ButtonSize, string> = {
  md: "h-7 px-2 gap-1 rounded-md text-body-sm font-medium",
  lg: "h-7 px-3 gap-1 rounded-lg text-body-sm font-medium",
};

const DISABLED_CLASSES: Record<ButtonDisabledStyle, string> = {
  grey: "disabled:bg-bg-lighter disabled:text-fg-moderate disabled:cursor-not-allowed disabled:hover:bg-bg-lighter",
  dim: "disabled:opacity-50 disabled:cursor-not-allowed",
};

const SPINNER_COLOR: Record<ButtonVariant, string> = {
  primary: "border-white",
  danger: "border-white",
  secondary: "border-current",
  ghost: "border-current",
};

export function Button({
  variant = "primary",
  size = "md",
  loading = false,
  disabledStyle = "grey",
  icon,
  trailingIcon,
  disabled,
  className = "",
  children,
  ...buttonProps
}: ButtonProps) {
  return (
    <button
      disabled={disabled || loading}
      className={`inline-flex items-center justify-center transition-colors ${VARIANT_CLASSES[variant]} ${SIZE_CLASSES[size]} ${DISABLED_CLASSES[disabledStyle]} ${className}`}
      {...buttonProps}
    >
      {loading ? <Spinner color={SPINNER_COLOR[variant]} /> : icon}
      {children}
      {trailingIcon}
    </button>
  );
}
