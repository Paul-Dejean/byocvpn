import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { useDisconnectReminderPreference } from "../../hooks/useDisconnectReminderPreference";
import { InfoBanner } from "../common/InfoBanner";

interface DisconnectReminderProps {
  instanceId: string;
}

export function DisconnectReminder({ instanceId }: DisconnectReminderProps) {
  const { recentlyDisconnectedInstanceId, clearRecentlyDisconnectedInstance } =
    useVpnConnectionContext();
  const { isMuted, recordDismissal, muteReminder } =
    useDisconnectReminderPreference();

  if (recentlyDisconnectedInstanceId !== instanceId || isMuted) {
    return null;
  }

  function onDismissReminder() {
    clearRecentlyDisconnectedInstance();
    recordDismissal();
  }

  function onMuteReminder() {
    clearRecentlyDisconnectedInstance();
    muteReminder();
  }

  return (
    <InfoBanner onDismiss={onDismissReminder}>
      Disconnecting VPN doesn't stop this server. Terminate it when done to
      stop charges.{" "}
      <button
        type="button"
        onClick={onMuteReminder}
        className="text-blue-300 underline hover:text-primary transition-colors"
      >
        Don't show again
      </button>
    </InfoBanner>
  );
}
