import { useEffect, useState } from "react";
import { invokeCommand } from "../../lib/invokeCommand";
import { Toggle } from "../primitives/Toggle";
import { DurationField, DurationUnit } from "./DurationField";

interface AutoTerminateSettings {
  autoTerminateEnabled: boolean;
  autoTerminateThresholdMinutes: number;
  autoTerminateUnit: DurationUnit;
}

const DEFAULT_SETTINGS: AutoTerminateSettings = {
  autoTerminateEnabled: false,
  autoTerminateThresholdMinutes: 720,
  autoTerminateUnit: "hours",
};

const MIN_THRESHOLD_MINUTES = 5;

export function AutoTerminateSettingsCard() {
  const [settings, setSettings] =
    useState<AutoTerminateSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    invokeCommand<AutoTerminateSettings>("get_auto_terminate_settings")
      .then(setSettings)
      .catch((error) =>
        console.error("Failed to load auto-terminate settings:", error),
      );
  }, []);

  const updateSettings = (updated: AutoTerminateSettings) => {
    setSettings(updated);
    invokeCommand("save_auto_terminate_settings", { settings: updated }).catch(
      (error) =>
        console.error("Failed to save auto-terminate settings:", error),
    );
  };

  const toggleEnabled = () => {
    updateSettings({
      ...settings,
      autoTerminateEnabled: !settings.autoTerminateEnabled,
    });
  };

  return (
    <div className="py-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 min-w-0">
          <div className="w-12 h-12 rounded-xl bg-danger-900/50 flex items-center justify-center flex-shrink-0">
            <ClockIcon />
          </div>
          <div className="min-w-0">
            <h3 className="font-semibold text-primary">
              Auto-Terminate Servers
            </h3>
            <p className="text-sm text-gray-400 mt-0.5">
              Automatically terminate servers that have been running too long
            </p>
          </div>
        </div>

        <Toggle
          checked={settings.autoTerminateEnabled}
          onChange={toggleEnabled}
          ariaLabel="Toggle auto-terminate"
        />
      </div>

      {settings.autoTerminateEnabled && (
        <div className="mt-3 pl-16 space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-400">Terminate after</span>
            <DurationField
              minutes={settings.autoTerminateThresholdMinutes}
              unit={settings.autoTerminateUnit}
              minMinutes={MIN_THRESHOLD_MINUTES}
              onChange={(minutes, unit) =>
                updateSettings({
                  ...settings,
                  autoTerminateThresholdMinutes: minutes,
                  autoTerminateUnit: unit,
                })
              }
            />
          </div>

          <p className="text-xs text-gray-500">
            Only runs while the app is open. Servers are fully terminated, so a
            forgotten one won't keep costing you.
          </p>
        </div>
      )}
    </div>
  );
}

function ClockIcon() {
  return (
    <svg
      className="w-5 h-5 text-danger-400"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}
