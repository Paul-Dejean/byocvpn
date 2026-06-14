import { ButtonHTMLAttributes, ReactNode } from "react";

interface SelectableCardProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
}

export function SelectableCard({
  className = "",
  children,
  ...buttonProps
}: SelectableCardProps) {
  return (
    <button
      className={`w-full text-left transition-all ${className}`}
      {...buttonProps}
    >
      {children}
    </button>
  );
}
