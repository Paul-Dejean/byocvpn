import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { useDisconnectReminderPreference } from "../../hooks/useDisconnectReminderPreference";
import { Banner } from "../primitives/Banner";

interface DisconnectReminderProps {
  instanceId: string;
}

export function DisconnectReminder({ instanceId }: DisconnectReminderProps) {
  const { recentlyDisconnectedInstanceId, clearRecentlyDisconnectedInstance } =
    useVpnConnectionContext();
  const { isMuted, recordDismissal, muteReminder } = useDisconnectReminderPreference();

  if (recentlyDisconnectedInstanceId !== instanceId || isMuted) {
    return null;
  }

  function onDismissReminder() {
    clearRecentlyDisconnectedInstance();
    void recordDismissal();
  }

  function onMuteReminder() {
    clearRecentlyDisconnectedInstance();
    void muteReminder();
  }

  return (
    <Banner variant="info" onDismiss={onDismissReminder}>
      Disconnecting VPN doesn't stop this server. Terminate it when done to stop charges.{" "}
      <button
        type="button"
        onClick={onMuteReminder}
        className="text-fg-brand underline transition-colors hover:text-fg-lighter"
      >
        Don't show again
      </button>
    </Banner>
  );
}
