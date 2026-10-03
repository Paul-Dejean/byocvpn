import { useEffect, useState } from "react";
import { Bell } from "lucide-react";
import { NotificationSettings, commands } from "../../bindings";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Banner } from "../primitives/Banner";
import { Toggle } from "../primitives/Toggle";
import { Button } from "../primitives/Button";
import { DurationField } from "./DurationField";
import { SettingsRow } from "./SettingsRow";

const DEFAULT_SETTINGS: NotificationSettings = {
  notificationEnabled: false,
  notificationThresholdMinutes: 60,
  notificationUnit: "minutes",
};

export function NotificationSettingsCard() {
  const [settings, setSettings] =
    useState<NotificationSettings>(DEFAULT_SETTINGS);
  const [permissionError, setPermissionError] = useState<string | null>(null);

  useEffect(() => {
    commands
      .getNotificationSettings()
      .then(setSettings)
      .catch((error) =>
        console.error("Failed to load notification settings:", error),
      );
  }, []);

  const updateSettings = (updated: NotificationSettings) => {
    setSettings(updated);
    commands.saveNotificationSettings(updated).then((result) => {
      if (result.status === "error") {
        console.error("Failed to save notification settings:", result.error);
      }
    });
  };

  const toggleEnabled = async () => {
    const enabling = !settings.notificationEnabled;

    if (enabling) {
      try {
        const alreadyGranted = await isPermissionGranted();
        if (!alreadyGranted) {
          const result = await requestPermission();
          if (result !== "granted") {
            setPermissionError(
              "Notification permission was denied. Enable it in System Settings.",
            );
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
    if (navigator.platform.startsWith("Mac"))
      return "x-apple.systempreferences:com.apple.preference.notifications";
    if (navigator.platform.startsWith("Win"))
      return "ms-settings:notifications";
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

  return (
    <SettingsRow
      icon={<Bell size={16} />}
      title="Server uptime notifications"
      description="Get notified when a server has been running too long"
      control={
        <Toggle
          checked={settings.notificationEnabled}
          onChange={toggleEnabled}
          ariaLabel="Toggle notifications"
        />
      }
    >
      {settings.notificationEnabled && (
        <>
          <div className="flex items-center gap-2 text-caption text-fg-medium">
            <span>Notify after</span>
            <DurationField
              minutes={settings.notificationThresholdMinutes}
              unit={settings.notificationUnit}
              minMinutes={1}
              onChange={(minutes, unit) =>
                updateSettings({
                  ...settings,
                  notificationThresholdMinutes: minutes,
                  notificationUnit: unit,
                })
              }
            />
            <span>of server uptime</span>
          </div>

          <p className="text-caption text-fg-moderate">
            To verify notifications work, open System Settings and allow
            notifications for this app, then send a test notification.
          </p>

          {permissionError && <Banner variant="danger">{permissionError}</Banner>}

          <div className="flex items-center gap-2">
            {notificationSettingsUrl && (
              <Button
                variant="secondary"
                size="md"
                onClick={openNotificationSettings}
              >
                Open Settings
              </Button>
            )}
            <Button
              variant="secondary"
              size="md"
              onClick={sendTestNotification}
            >
              Test notification
            </Button>
          </div>
        </>
      )}
    </SettingsRow>
  );
}
