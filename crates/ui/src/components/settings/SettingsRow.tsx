import { ReactNode } from "react";

interface SettingsRowProps {
  icon: ReactNode;
  title: string;
  description: string;
  control?: ReactNode;
  children?: ReactNode;
}

export function SettingsRow({
  icon,
  title,
  description,
  control,
  children,
}: SettingsRowProps) {
  return (
    <div className="p-4 flex flex-col gap-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className="w-9 h-9 rounded-lg bg-gray-700 border border-gray-500/60 flex items-center justify-center flex-shrink-0 text-gray-200">
            {icon}
          </div>
          <div className="min-w-0 flex flex-col gap-0.5">
            <h3 className="text-sm text-primary">{title}</h3>
            <p className="text-xs text-gray-300">{description}</p>
          </div>
        </div>
        {control}
      </div>
      {children && <div className="pl-12 flex flex-col gap-3">{children}</div>}
    </div>
  );
}
