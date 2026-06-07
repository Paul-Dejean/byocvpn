import { useEffect, useState } from "react";
import { invokeCommand } from "../../lib/invokeCommand";

interface KillSwitchSettings {
  killSwitchEnabled: boolean;
}

export function KillSwitchSettingsCard() {
  const [killSwitchEnabled, setKillSwitchEnabled] = useState(false);

  useEffect(() => {
    invokeCommand<KillSwitchSettings>("get_kill_switch_settings")
      .then((settings) => setKillSwitchEnabled(settings.killSwitchEnabled))
      .catch((error) => console.error("Failed to load kill switch settings:", error));
  }, []);

  const toggleKillSwitch = () => {
    const enabled = !killSwitchEnabled;
    setKillSwitchEnabled(enabled);
    invokeCommand("save_kill_switch_settings", { enabled }).catch((error) => {
      console.error("Failed to save kill switch settings:", error);
      setKillSwitchEnabled(!enabled);
    });
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-red-900/50 flex items-center justify-center flex-shrink-0">
            <ShieldIcon />
          </div>
          <div>
            <h3 className="text-sm font-medium text-white">Kill Switch</h3>
            <p className="text-xs text-gray-400 mt-0.5">
              Block all traffic if the VPN tunnel drops
            </p>
          </div>
        </div>
        <button
          onClick={toggleKillSwitch}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
            killSwitchEnabled ? "bg-blue-600" : "bg-gray-600"
          }`}
          aria-label="Toggle kill switch"
        >
          <span
            className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
              killSwitchEnabled ? "translate-x-4.5" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>
    </div>
  );
}

function ShieldIcon() {
  return (
    <svg
      className="w-4 h-4 text-red-400"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
      />
    </svg>
  );
}
