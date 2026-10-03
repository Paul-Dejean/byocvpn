import { ReactNode } from "react";

interface SettingsSectionProps {
  title: string;
  description?: string;
  children: ReactNode;
}

export function SettingsSection({
  title,
  description,
  children,
}: SettingsSectionProps) {
  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-col gap-0.5">
        <h2 className="text-sm font-medium text-primary">{title}</h2>
        {description && <p className="text-xs text-gray-300">{description}</p>}
      </div>
      <div className="rounded-xl bg-gray-750 border border-gray-500/50 divide-y divide-gray-500/40">
        {children}
      </div>
    </section>
  );
}
