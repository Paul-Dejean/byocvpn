import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";

interface NotificationSettings {
  notificationEnabled: boolean;
  notificationThresholdMinutes: number;
}

const DEFAULT_SETTINGS: NotificationSettings = {
  notificationEnabled: false,
  notificationThresholdMinutes: 60,
};

export function NotificationSettingsCard() {
  const [settings, setSettings] = useState<NotificationSettings>(DEFAULT_SETTINGS);
  const [permissionError, setPermissionError] = useState<string | null>(null);

  useEffect(() => {
    invoke<NotificationSettings>("get_notification_settings")
      .then(setSettings)
      .catch((error) => console.error("Failed to load notification settings:", error));
  }, []);

  const updateSettings = (updated: NotificationSettings) => {
    setSettings(updated);
    invoke("save_notification_settings", { settings: updated }).catch((error) =>
      console.error("Failed to save notification settings:", error),
    );
  };

  const toggleEnabled = async () => {
    const enabling = !settings.notificationEnabled;

    if (enabling) {
      try {
        const alreadyGranted = await isPermissionGranted();
        if (!alreadyGranted) {
          const result = await requestPermission();
          if (result !== "granted") {
            setPermissionError("Notification permission was denied. Enable it in System Settings.");
            return;
          }
        }
        setPermissionError(null);
      } catch (error) {
        console.error("Could not check notification permission:", error);
        setPermissionError("Could not request notification permission.");
        return;
      }
    }

    updateSettings({ ...settings, notificationEnabled: enabling });
  };

  const onThresholdChange = (raw: string) => {
    const minutes = parseInt(raw, 10);
    if (!isNaN(minutes) && minutes >= 1) {
      updateSettings({ ...settings, notificationThresholdMinutes: minutes });
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-yellow-900/50 flex items-center justify-center flex-shrink-0">
            <BellIcon />
          </div>
          <div>
            <h3 className="text-sm font-medium text-white">Server Uptime Notifications</h3>
            <p className="text-xs text-gray-400 mt-0.5">
              Get notified when a server has been running too long
            </p>
          </div>
        </div>
        <button
          onClick={toggleEnabled}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
            settings.notificationEnabled ? "bg-blue-600" : "bg-gray-600"
          }`}
          aria-label="Toggle notifications"
        >
          <span
            className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
              settings.notificationEnabled ? "translate-x-4.5" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>

      {permissionError && (
        <p className="mt-2 text-xs text-red-400">{permissionError}</p>
      )}

      {settings.notificationEnabled && (
        <div className="mt-4 pt-4 border-t border-gray-700">
          <label className="text-xs text-gray-400 mb-2 block">Notify after (minutes)</label>
          <input
            type="number"
            min={1}
            value={settings.notificationThresholdMinutes}
            onChange={(event) => onThresholdChange(event.target.value)}
            className="w-24 px-3 py-1.5 text-xs bg-gray-700 text-white rounded-md border border-gray-600 focus:outline-none focus:border-blue-500"
          />
        </div>
      )}
    </div>
  );
}

function BellIcon() {
  return (
    <svg
      className="w-4 h-4 text-yellow-400"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
      />
    </svg>
  );
}
