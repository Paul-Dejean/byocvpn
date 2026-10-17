import { ReactNode } from "react";

type StatusTone = "protected" | "unprotected";

interface StatusHeroProps {
  tone: StatusTone;
  icon: ReactNode;
  title: string;
  children?: ReactNode;
}

const ICON_BACKGROUND_CLASSES: Record<StatusTone, string> = {
  protected: "bg-bg-success-faint/70",
  unprotected: "bg-bg-danger-faint/70",
};

export function StatusHero({ tone, icon, title, children }: StatusHeroProps) {
  return (
    <div className="flex flex-col items-center gap-6 rounded-md px-2.5 py-6">
      <div
        className={`flex size-24 items-center justify-center rounded-full p-6 ${ICON_BACKGROUND_CLASSES[tone]}`}
      >
        {icon}
      </div>
      <div className="flex flex-col items-center gap-2 text-center">
        <h2 className="text-highlight font-semibold text-fg-lighter">{title}</h2>
        {children}
      </div>
    </div>
  );
}
