import { ReactNode } from "react";

interface CardProps {
  children: ReactNode;
  padded?: boolean;
  className?: string;
}

export function Card({ children, padded = true, className = "" }: CardProps) {
  return (
    <div className={`bg-bg-strong/60 rounded-lg ${padded ? "p-6" : ""} ${className}`}>
      {children}
    </div>
  );
}
