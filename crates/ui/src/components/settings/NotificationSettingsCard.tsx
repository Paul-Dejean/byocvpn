import { useEffect, useState } from "react";
import { invokeCommand } from "../../lib/invokeCommand";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { openUrl } from "@tauri-apps/plugin-opener";

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
    invokeCommand<NotificationSettings>("get_notification_settings")
      .then(setSettings)
      .catch((error) => console.error("Failed to load notification settings:", error));
  }, []);

  const updateSettings = (updated: NotificationSettings) => {
    setSettings(updated);
    invokeCommand("save_notification_settings", { settings: updated }).catch((error) =>
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

  const sendTestNotification = () => {
    sendNotification({
      title: "ByocVPN",
      body: "Notifications are working!",
    });
  };

  const notificationSettingsUrl = (() => {
    if (navigator.platform.startsWith("Mac")) return "x-apple.systempreferences:com.apple.preference.notifications";
    if (navigator.platform.startsWith("Win")) return "ms-settings:notifications";
    return null;
  })();

  const openNotificationSettings = async () => {
    if (!notificationSettingsUrl) return;
    try {
      await openUrl(notificationSettingsUrl);
    } catch (error) {
      console.error("Failed to open notification settings:", error);
      setPermissionError(`Could not open System Settings: ${error}`);
    }
  };

  const onThresholdChange = (raw: string) => {
    const minutes = parseInt(raw, 10);
    if (!isNaN(minutes) && minutes >= 1) {
      updateSettings({ ...settings, notificationThresholdMinutes: minutes });
    }
  };

  return (
    <div className="py-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 min-w-0">
          <div className="w-12 h-12 rounded-xl bg-yellow-900/50 flex items-center justify-center flex-shrink-0">
            <BellIcon />
          </div>
          <div className="min-w-0">
            <h3 className="font-semibold text-white">Server Uptime Notifications</h3>
            <p className="text-sm text-gray-400 mt-0.5">
              Get notified when a server has been running too long
            </p>
          </div>
        </div>

        <button
          onClick={toggleEnabled}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors flex-shrink-0 ${
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

      {settings.notificationEnabled && (
        <div className="mt-3 pl-16 space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-400">Notify after</span>
            <input
              type="number"
              min={1}
              value={settings.notificationThresholdMinutes}
              onChange={(event) => onThresholdChange(event.target.value)}
              className="w-14 px-2 py-1 text-xs bg-gray-700 text-white rounded-md border border-gray-600 focus:outline-none focus:border-blue-500 text-center"
            />
            <span className="text-xs text-gray-400">minutes of server uptime</span>
          </div>

          <p className="text-xs text-gray-500">
            To verify notifications work, open System Settings and allow notifications for this app. You can then send a test notification to confirm everything is set up correctly.
          </p>

          {permissionError && (
            <p className="text-xs text-red-400">{permissionError}</p>
          )}

          <div className="flex items-center gap-2">
            {notificationSettingsUrl && (
              <button
                onClick={openNotificationSettings}
                className="text-xs text-gray-400 hover:text-white border border-gray-600 hover:border-gray-400 rounded-md px-2.5 py-1 transition-colors"
              >
                Open Settings
              </button>
            )}
            <button
              onClick={sendTestNotification}
              className="text-xs text-gray-400 hover:text-white border border-gray-600 hover:border-gray-400 rounded-md px-2.5 py-1 transition-colors"
            >
              Test Notification
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function BellIcon() {
  return (
    <svg
      className="w-5 h-5 text-yellow-400"
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
