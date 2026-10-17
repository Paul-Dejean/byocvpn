import { ReactNode } from "react";

interface SettingsRowProps {
  icon: ReactNode;
  title: string;
  description: string;
  control?: ReactNode;
  children?: ReactNode;
}

export function SettingsRow({ icon, title, description, control, children }: SettingsRowProps) {
  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-bd-moderate bg-bg-medium text-fg-medium">
            {icon}
          </div>
          <div className="flex min-w-0 flex-col gap-0.5">
            <h3 className="text-body-sm text-fg-lighter">{title}</h3>
            <p className="text-caption text-fg-medium">{description}</p>
          </div>
        </div>
        {control}
      </div>
      {children && <div className="flex flex-col gap-3 pl-12">{children}</div>}
    </div>
  );
}
