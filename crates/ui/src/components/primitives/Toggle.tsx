interface ToggleProps {
  checked: boolean;
  onChange: () => void;
  ariaLabel: string;
}

export function Toggle({ checked, onChange, ariaLabel }: ToggleProps) {
  return (
    <button
      onClick={onChange}
      className={`relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors ${
        checked ? "bg-bg-brand-bolder" : "bg-bg-lighter"
      }`}
      aria-label={ariaLabel}
    >
      <span
        className={`inline-block size-3.5 transform rounded-full bg-fg-on-brand transition-transform ${
          checked ? "translate-x-4.5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}
