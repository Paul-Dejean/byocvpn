import { useEffect, useState } from "react";
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

export function DurationField({
  minutes,
  unit,
  minMinutes,
  onChange,
}: DurationFieldProps) {
  const displayedValue = Math.round(minutes / unitMultiplier(unit));
  const [inputValue, setInputValue] = useState(String(displayedValue));

  useEffect(() => {
    setInputValue(String(displayedValue));
  }, [displayedValue]);

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
    const nextMinutes = Math.max(
      displayedValue * unitMultiplier(nextUnit),
      minMinutes,
    );
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
        className="w-14 px-2 py-1 text-caption bg-bg-medium text-fg-lighter rounded-md border border-bd-moderate focus:outline-none focus:border-bd-brand text-center"
      />
      <div className="inline-flex rounded-md border border-bd-moderate overflow-hidden">
        {(["minutes", "hours"] as DurationUnit[]).map((option) => (
          <button
            key={option}
            type="button"
            onClick={() => onUnitChange(option)}
            className={`px-2.5 py-1 text-caption capitalize transition-colors ${
              unit === option
                ? "bg-bg-brand-bolder text-white"
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
