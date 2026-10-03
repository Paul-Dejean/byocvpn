import { ReactNode } from "react";

type StatusTone = "protected" | "unprotected";

interface StatusHeroProps {
  tone: StatusTone;
  icon: ReactNode;
  title: string;
  children?: ReactNode;
}

const ICON_BACKGROUND_CLASSES: Record<StatusTone, string> = {
  protected: "bg-bg-success-faint",
  unprotected: "bg-bg-danger-faint",
};

export function StatusHero({ tone, icon, title, children }: StatusHeroProps) {
  return (
    <div className="flex flex-col items-center gap-6 py-6 px-2.5 rounded-md">
      <div
        className={`w-24 h-24 rounded-full p-6 flex items-center justify-center opacity-70 ${ICON_BACKGROUND_CLASSES[tone]}`}
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
