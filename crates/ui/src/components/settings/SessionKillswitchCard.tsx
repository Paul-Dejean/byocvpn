import { useEffect, useState } from "react";
import { invokeCommand } from "../../lib/invokeCommand";

interface VpnSettings {
  sessionKillswitch: boolean;
}

const DEFAULT_SETTINGS: VpnSettings = {
  sessionKillswitch: true,
};

export function SessionKillswitchCard() {
  const [settings, setSettings] = useState<VpnSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    invokeCommand<VpnSettings>("get_vpn_settings")
      .then(setSettings)
      .catch((error) => console.error("Failed to load VPN settings:", error));
  }, []);

  const toggleKillswitch = () => {
    const updated: VpnSettings = {
      ...settings,
      sessionKillswitch: !settings.sessionKillswitch,
    };
    setSettings(updated);
    invokeCommand("save_vpn_settings", { settings: updated }).catch((error) =>
      console.error("Failed to save VPN settings:", error),
    );
  };

  return (
    <div className="py-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 min-w-0">
          <div className="w-12 h-12 rounded-xl bg-green-900/50 flex items-center justify-center flex-shrink-0">
            <ShieldIcon />
          </div>
          <div className="min-w-0">
            <h3 className="font-semibold text-white">Session Kill Switch</h3>
            <p className="text-sm text-gray-400 mt-0.5">
              Blocks all internet traffic that isn't going through the VPN tunnel
            </p>
          </div>
        </div>

        <button
          onClick={toggleKillswitch}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors flex-shrink-0 ${
            settings.sessionKillswitch ? "bg-blue-600" : "bg-gray-600"
          }`}
          aria-label="Toggle session kill switch"
        >
          <span
            className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
              settings.sessionKillswitch ? "translate-x-4.5" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>

      <div className="mt-3 pl-16">
        <p className="text-xs text-gray-500">
          Only allows the VPN tunnel and local traffic while connected, so your real IP address is
          never leaked. But if the tunnel drops, all other traffic is blocked, which can lock you
          out of the internet until you reconnect or disconnect.
        </p>
      </div>
    </div>
  );
}

function ShieldIcon() {
  return (
    <svg className="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M9 12.75L11.25 15 15 9.75M21 12c0 5.25-3.75 8.25-8.567 9.674a.75.75 0 01-.366 0C7.25 20.25 3.5 17.25 3.5 12V6.75a.75.75 0 01.44-.683 12.75 12.75 0 008.06-2.06.75.75 0 01.75 0 12.75 12.75 0 008.06 2.06.75.75 0 01.44.683V12z"
      />
    </svg>
  );
}
