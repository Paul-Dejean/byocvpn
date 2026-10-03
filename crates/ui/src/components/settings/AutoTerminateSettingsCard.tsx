import { useEffect, useState } from "react";
import { Clock } from "lucide-react";
import { AutoTerminateSettings, commands } from "../../bindings";
import { Toggle } from "../primitives/Toggle";
import { DurationField } from "./DurationField";
import { SettingsRow } from "./SettingsRow";

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
    commands
      .getAutoTerminateSettings()
      .then(setSettings)
      .catch((error) =>
        console.error("Failed to load auto-terminate settings:", error),
      );
  }, []);

  const updateSettings = (updated: AutoTerminateSettings) => {
    setSettings(updated);
    commands.saveAutoTerminateSettings(updated).then((result) => {
      if (result.status === "error") {
        console.error("Failed to save auto-terminate settings:", result.error);
      }
    });
  };

  const toggleEnabled = () => {
    updateSettings({
      ...settings,
      autoTerminateEnabled: !settings.autoTerminateEnabled,
    });
  };

  return (
    <SettingsRow
      icon={<Clock size={16} />}
      title="Auto-terminate idle servers"
      description="Automatically terminate servers left running without a VPN connection"
      control={
        <Toggle
          checked={settings.autoTerminateEnabled}
          onChange={toggleEnabled}
          ariaLabel="Toggle auto-terminate"
        />
      }
    >
      {settings.autoTerminateEnabled && (
        <>
          <div className="flex items-center gap-2 text-caption text-fg-medium">
            <span>Terminate after</span>
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
            <span>without a connection</span>
          </div>
          <p className="text-caption text-fg-moderate">
            The idle timer resets every time you connect. Only runs while the
            app is open. Servers are fully terminated, so a forgotten one won't
            keep costing you.
          </p>
        </>
      )}
    </SettingsRow>
  );
}
