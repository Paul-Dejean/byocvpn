import { useState, useEffect } from "react";
import { Page } from "../App";
import { useDaemonInstaller } from "../hooks/useDaemonInstaller";
import { useProfile } from "../hooks";

interface DaemonSetupPageProps {
  setPage: (page: Page) => void;
}

export function DaemonSetupPage({ setPage }: DaemonSetupPageProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [showManualInstructions, setShowManualInstructions] = useState(false);
  const { isInstalling, error, installDaemon, clearError } =
    useDaemonInstaller();
  const { checkProfile } = useProfile();

  useEffect(() => {
    setIsVisible(true);
  }, []);

  const handleInstall = async () => {
    clearError();
    const installed = await installDaemon();
    if (!installed) return;

    const hasProfile = await checkProfile();
    setPage(hasProfile ? Page.VPN : Page.SETUP);
  };

  return (
    <div className="relative bg-[url('/landing-page-bg.png')] bg-cover bg-center h-screen flex items-center justify-center overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-br from-black/60 to-transparent" />

      <div
        className={`relative z-10 flex flex-col items-center max-w-lg mx-auto px-6 text-center transition-opacity duration-700 ${isVisible ? "opacity-100" : "opacity-0"}`}
      >
        {/* Icon */}
        <div className="w-16 h-16 rounded-full bg-blue-500/20 backdrop-blur-sm flex items-center justify-center mb-8 border border-blue-400/30">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-8 w-8 text-blue-300"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
            />
          </svg>
        </div>

        <h1 className="text-4xl font-bold text-white mb-4">One-time setup</h1>

        <p className="text-blue-100 text-lg mb-3">
          BYOC VPN needs a background service to manage your VPN tunnel.
        </p>
        <p className="text-gray-400 text-sm mb-8">
          You'll be asked for your Mac password once to install it. After that,
          it starts automatically on boot and you'll never need to do this
          again.
        </p>

        {/* What it does */}
        <div className="w-full bg-white/5 backdrop-blur-md rounded-xl p-5 mb-8 text-left border border-white/10">
          <p className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
            What gets installed
          </p>
          <ul className="space-y-2 text-sm text-gray-300">
            <li className="flex items-start gap-2">
              <span className="text-blue-400 mt-0.5">→</span>
              <span>
                <code className="text-blue-300">
                  /Library/PrivilegedHelperTools/byocvpn-daemon
                </code>{" "}
                — background service
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-blue-400 mt-0.5">→</span>
              <span>
                <code className="text-blue-300">
                  /Library/LaunchDaemons/com.byocvpn.daemon.plist
                </code>{" "}
                — auto-start config
              </span>
            </li>
          </ul>
        </div>

        {error && (
          <div className="w-full bg-red-500/10 border border-red-500/30 rounded-lg px-4 py-3 mb-5 text-sm text-red-300 text-left">
            {error}
          </div>
        )}

        {/* Primary action */}
        <button
          onClick={handleInstall}
          disabled={isInstalling}
          className="w-full px-8 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-all duration-300 font-medium flex items-center justify-center shadow-lg shadow-blue-600/30 disabled:opacity-50 disabled:cursor-not-allowed mb-3"
        >
          {isInstalling ? (
            <>
              <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
              Installing…
            </>
          ) : (
            <>
              Install background service
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5 ml-2"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fillRule="evenodd"
                  d="M12.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-2.293-2.293a1 1 0 010-1.414z"
                  clipRule="evenodd"
                />
              </svg>
            </>
          )}
        </button>

        <button
          onClick={() => setShowManualInstructions((previous) => !previous)}
          className="text-sm text-gray-500 hover:text-gray-300 transition-colors mb-3"
        >
          {showManualInstructions
            ? "Hide"
            : "Install manually with a terminal instead"}
        </button>

        {showManualInstructions && (
          <div className="w-full bg-black/40 rounded-xl p-4 text-left border border-white/10 text-xs font-mono text-green-300 space-y-1">
            <p className="text-gray-400 font-sans text-xs mb-2">
              Run this in your terminal:
            </p>
            <p>sudo ./install.sh</p>
            <p className="text-gray-500 mt-2">Then relaunch the app.</p>
          </div>
        )}
      </div>
    </div>
  );
}
