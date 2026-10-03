interface FormFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  error?: string;
  type?: "text" | "password" | "number";
  mono?: boolean;
  multiline?: boolean;
  placeholder?: string;
  rows?: number;
}

export function FormField({
  label,
  value,
  onChange,
  hint,
  error,
  type = "text",
  mono = false,
  multiline = false,
  placeholder,
  rows = 4,
}: FormFieldProps) {
  const inputClasses = `input ${mono ? "font-mono text-xs" : ""}`;

  return (
    <div className="flex flex-col gap-1.5">
      <label className="text-xs text-gray-200">{label}</label>
      {multiline ? (
        <textarea
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={placeholder}
          rows={rows}
          className={`${inputClasses} resize-none`}
        />
      ) : (
        <input
          type={type}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={placeholder ?? hint}
          className={inputClasses}
        />
      )}
      {hint && placeholder && <p className="text-xs text-gray-400">{hint}</p>}
      {error && <p className="text-xs text-danger-400">{error}</p>}
    </div>
  );
}
