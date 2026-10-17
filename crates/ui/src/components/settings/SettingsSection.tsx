import { ReactNode } from "react";

interface SettingsSectionProps {
  title: string;
  description?: string;
  children: ReactNode;
}

export function SettingsSection({ title, description, children }: SettingsSectionProps) {
  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-col gap-0.5">
        <h2 className="text-body-sm font-medium text-fg-lighter">{title}</h2>
        {description && <p className="text-caption text-fg-medium">{description}</p>}
      </div>
      <div className="divide-y divide-bd-faint rounded-xl border border-bd-moderate bg-bg-bolder">
        {children}
      </div>
    </section>
  );
}
