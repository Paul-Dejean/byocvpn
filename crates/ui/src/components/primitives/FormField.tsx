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
  const inputClasses = `input ${mono ? "font-mono text-sm" : ""}`;

  return (
    <div>
      <label className="block text-sm font-medium text-gray-300 mb-1">{label}</label>
      {hint && <p className="text-xs text-gray-500 mb-2">{hint}</p>}
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
          placeholder={placeholder}
          className={inputClasses}
        />
      )}
      {error && <p className="text-xs text-danger-400 mt-1">{error}</p>}
    </div>
  );
}
