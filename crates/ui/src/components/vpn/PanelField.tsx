interface PanelFieldProps {
  label: string;
  value: string;
  mono?: boolean;
}

export function PanelField({ label, value, mono = false }: PanelFieldProps) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-xs text-gray-300">{label}</span>
      <span
        className={`text-sm text-primary truncate ${mono ? "font-mono" : "tabular-nums"}`}
        title={value}
      >
        {value}
      </span>
    </div>
  );
}
