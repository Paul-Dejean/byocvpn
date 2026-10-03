interface PanelFieldProps {
  label: string;
  value: string;
  wrap?: boolean;
}

export function PanelField({ label, value, wrap = false }: PanelFieldProps) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-caption text-fg-medium">{label}</span>
      <span
        className={`text-body-sm text-fg-lighter tabular-nums ${wrap ? "break-all" : "truncate"}`}
        title={value}
      >
        {value}
      </span>
    </div>
  );
}
