import { useState } from "react";
import type { DurationUnit } from "../../bindings";

interface DurationFieldProps {
  minutes: number;
  unit: DurationUnit;
  minMinutes: number;
  onChange: (minutes: number, unit: DurationUnit) => void;
}

function unitMultiplier(unit: DurationUnit): number {
  return unit === "hours" ? 60 : 1;
}

export function DurationField({ minutes, unit, minMinutes, onChange }: DurationFieldProps) {
  const displayedValue = Math.round(minutes / unitMultiplier(unit));
  const [inputValue, setInputValue] = useState(String(displayedValue));
  const [previousDisplayedValue, setPreviousDisplayedValue] = useState(displayedValue);

  if (previousDisplayedValue !== displayedValue) {
    setPreviousDisplayedValue(displayedValue);
    setInputValue(String(displayedValue));
  }

  const minDisplayed = unit === "hours" ? 1 : minMinutes;

  const onValueChange = (raw: string) => {
    setInputValue(raw);
    const value = parseInt(raw, 10);
    const nextMinutes = value * unitMultiplier(unit);
    if (!isNaN(value) && nextMinutes >= minMinutes) {
      onChange(nextMinutes, unit);
    }
  };

  const onBlur = () => {
    setInputValue(String(displayedValue));
  };

  const onUnitChange = (nextUnit: DurationUnit) => {
    const nextMinutes = Math.max(displayedValue * unitMultiplier(nextUnit), minMinutes);
    onChange(nextMinutes, nextUnit);
  };

  return (
    <div className="flex items-center gap-2">
      <input
        type="number"
        min={minDisplayed}
        value={inputValue}
        onChange={(event) => onValueChange(event.target.value)}
        onBlur={onBlur}
        className="w-14 rounded-md border border-bd-moderate bg-bg-medium px-2 py-1 text-center text-caption text-fg-lighter outline-none focus:border-bd-brand"
      />
      <div className="inline-flex overflow-hidden rounded-md border border-bd-moderate">
        {(["minutes", "hours"] as DurationUnit[]).map((option) => (
          <button
            key={option}
            type="button"
            onClick={() => onUnitChange(option)}
            className={`px-2.5 py-1 text-caption capitalize transition-colors ${
              unit === option
                ? "bg-bg-brand-bolder text-fg-on-brand"
                : "bg-bg-medium text-fg-moderate hover:text-fg-lighter"
            }`}
          >
            {option}
          </button>
        ))}
      </div>
    </div>
  );
}
