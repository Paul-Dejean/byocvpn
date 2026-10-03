import { ReactNode } from "react";

export type TagTone = "neutral" | "brand" | "success" | "danger" | "warning";

interface TagProps {
  tone?: TagTone;
  dot?: boolean;
  children: ReactNode;
}

const TONE_CLASSES: Record<TagTone, string> = {
  neutral: "bg-bg-lighter text-fg-lighter",
  brand: "bg-bg-brand-faint text-fg-brand",
  success: "bg-bg-success-faint text-fg-success-moderate",
  danger: "bg-bg-danger-faint text-fg-danger-moderate",
  warning: "bg-bg-warning-faint text-fg-warning-moderate",
};

export function Tag({ tone = "neutral", dot = false, children }: TagProps) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 h-4 px-2 rounded-full text-caption whitespace-nowrap ${TONE_CLASSES[tone]}`}
    >
      {dot && <span className="w-1.5 h-1.5 rounded-full bg-current" />}
      {children}
    </span>
  );
}
