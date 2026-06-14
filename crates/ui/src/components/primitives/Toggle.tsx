interface ToggleProps {
  checked: boolean;
  onChange: () => void;
  ariaLabel: string;
}

export function Toggle({ checked, onChange, ariaLabel }: ToggleProps) {
  return (
    <button
      onClick={onChange}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors flex-shrink-0 ${
        checked ? "bg-blue-600" : "bg-gray-600"
      }`}
      aria-label={ariaLabel}
    >
      <span
        className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
          checked ? "translate-x-4.5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}
