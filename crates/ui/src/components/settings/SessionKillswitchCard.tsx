import { useEffect, useState } from "react";
import { ShieldCheck } from "lucide-react";
import { VpnSettings, commands } from "../../bindings";
import { isAndroid } from "../../lib/platform";
import { Toggle } from "../primitives/Toggle";
import { SettingsRow } from "./SettingsRow";

const DEFAULT_SETTINGS: VpnSettings = {
  sessionKillswitch: true,
};

export function SessionKillswitchCard() {
  if (isAndroid()) {
    return <AndroidKillswitchRow />;
  }
  return <DesktopKillswitchRow />;
}

function DesktopKillswitchRow() {
  const [settings, setSettings] = useState<VpnSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    commands
      .getVpnSettings()
      .then(setSettings)
      .catch((error) => console.error("Failed to load VPN settings:", error));
  }, []);

  const toggleKillswitch = () => {
    const updated: VpnSettings = {
      ...settings,
      sessionKillswitch: !settings.sessionKillswitch,
    };
    setSettings(updated);
    commands.saveVpnSettings(updated).then((result) => {
      if (result.status === "error") {
        console.error("Failed to save VPN settings:", result.error);
      }
    });
  };

  return (
    <SettingsRow
      icon={<ShieldCheck size={16} />}
      title="Session kill switch"
      description="Blocks all internet traffic that isn't going through the VPN tunnel"
      control={
        <Toggle
          checked={settings.sessionKillswitch}
          onChange={toggleKillswitch}
          ariaLabel="Toggle session kill switch"
        />
      }
    >
      <p className="text-caption text-fg-moderate">
        Only allows the VPN tunnel and local traffic while connected, so your
        real IP address is never leaked. If the tunnel drops, all other traffic
        is blocked until you reconnect or disconnect.
      </p>
    </SettingsRow>
  );
}

function AndroidKillswitchRow() {
  return (
    <SettingsRow
      icon={<ShieldCheck size={16} />}
      title="Kill switch"
      description="Managed by Android"
    >
      <p className="text-caption text-fg-moderate">
        To block all traffic when the tunnel is down, open Android Settings ›
        Network & internet › VPN, tap the gear next to ByocVPN, and turn on
        both "Always-on VPN" and "Block connections without VPN".
      </p>
    </SettingsRow>
  );
}
