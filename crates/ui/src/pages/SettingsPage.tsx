import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCredentials } from "../hooks/useCredentials";
import { OracleProfileCard } from "../components/settings/OracleProfileCard";
import { GcpProfileCard } from "../components/settings/GcpProfileCard";
import { AzureProfileCard } from "../components/settings/AzureProfileCard";

interface SettingsPageProps {
  onNavigateBack?: () => void;
}

export function SettingsPage({ onNavigateBack }: SettingsPageProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [isUninstallConfirming, setIsUninstallConfirming] = useState(false);
  const [isUninstalling, setIsUninstalling] = useState(false);
  const [uninstallError, setUninstallError] = useState<string | null>(null);

  const handleUninstall = async () => {
    setIsUninstalling(true);
    setUninstallError(null);
    try {
      await invoke("uninstall_app");
      await getCurrentWindow().close();
    } catch (error) {
      setUninstallError(String(error));
      setIsUninstalling(false);
      setIsUninstallConfirming(false);
    }
  };
  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");

  const { isSaving, error, successMessage, saveCredentials, loadCredentials } =
    useCredentials();

  const handleEditProfile = async () => {
    const existing = await loadCredentials("aws");
    if (existing) {
      setAccessKey((existing as { accessKeyId: string }).accessKeyId);

    }
    setIsEditing(true);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setAccessKey("");
    setSecretKey("");
  };

  const handleSaveProfile = async () => {
    if (!accessKey.trim()) {
      return;
    }

    const success = await saveCredentials("aws", {
      accessKeyId: accessKey.trim(),
      secretAccessKey: secretKey.trim(),
    });

    if (success) {
      setAccessKey("");
      setSecretKey("");
      setIsEditing(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
      {}
      <div className="bg-gray-800 p-6 border-b border-gray-700 flex-shrink-0">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            {onNavigateBack && (
              <button
                onClick={onNavigateBack}
                className="p-2 rounded-lg bg-gray-700 hover:bg-gray-600 transition-colors"
                title="Back to VPN"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6 text-gray-300"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 19l-7-7 7-7"
                  />
                </svg>
              </button>
            )}
            <div>
              <h1 className="text-3xl font-bold mb-2 text-blue-400">
                Settings
              </h1>
              <p className="text-gray-300">
                Manage your cloud provider profiles
              </p>
            </div>
          </div>
        </div>
      </div>

      {}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto">
          {}
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-6 text-blue-400">
              Manage Profiles
            </h2>

            {}
            <div className="bg-gray-700 rounded-lg p-6">
              {!isEditing ? (

                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 bg-orange-600 rounded-lg flex items-center justify-center">
                      <svg
                        className="w-6 h-6 text-white"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M7.25 0C3.25 0 0 3.25 0 7.25v9.5C0 20.75 3.25 24 7.25 24h9.5c4 0 7.25-3.25 7.25-7.25v-9.5C24 3.25 20.75 0 16.75 0h-9.5zM12 6c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6 2.69-6 6-6z" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-lg text-white">
                        AWS Profile
                      </h3>
                      <p className="text-sm text-gray-400">
                        Amazon Web Services credentials for EC2 deployment
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={handleEditProfile}
                    className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition font-medium flex items-center gap-2"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                      />
                    </svg>
                    Edit
                  </button>
                </div>
              ) : (

                <div className="space-y-6">
                  <div className="flex items-center gap-4 mb-6">
                    <div className="w-12 h-12 bg-orange-600 rounded-lg flex items-center justify-center">
                      <svg
                        className="w-6 h-6 text-white"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M7.25 0C3.25 0 0 3.25 0 7.25v9.5C0 20.75 3.25 24 7.25 24h9.5c4 0 7.25-3.25 7.25-7.25v-9.5C24 3.25 20.75 0 16.75 0h-9.5zM12 6c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6 2.69-6 6-6z" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-lg text-white">
                        Edit AWS Profile
                      </h3>
                      <p className="text-sm text-gray-400">
                        Update your AWS access credentials
                      </p>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">
                        Access Key ID
                      </label>
                      <p className="text-xs text-gray-500 mb-2">
                        e.g. AKIAIOSFODNN7EXAMPLE
                      </p>
                      <input
                        type="text"
                        value={accessKey}
                        onChange={(e) => setAccessKey(e.target.value)}
                        className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-1">
                        Secret Access Key
                      </label>
                      <p className="text-xs text-gray-500 mb-2">
                        Leave blank to keep your existing key
                      </p>
                      <input
                        type="password"
                        value={secretKey}
                        onChange={(e) => setSecretKey(e.target.value)}
                        className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                      />
                    </div>

                    {}
                    {error && (
                      <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                        <p className="text-red-300 text-sm">{error}</p>
                      </div>
                    )}

                    {}
                    {successMessage && (
                      <div className="p-3 bg-green-900 border border-green-700 rounded-lg">
                        <p className="text-green-300 text-sm">
                          {successMessage}
                        </p>
                      </div>
                    )}

                    <div className="flex gap-3 pt-4">
                      <button
                        onClick={handleCancelEdit}
                        className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition font-medium"
                      >
                        Cancel
                      </button>
                      <button
                        onClick={handleSaveProfile}
                        disabled={isSaving || !accessKey.trim()}
                        className={`flex-1 px-4 py-2 rounded-lg transition font-medium ${
                          isSaving || !accessKey.trim()
                            ? "bg-gray-600 text-gray-400 cursor-not-allowed"
                            : "bg-blue-600 hover:bg-blue-700 text-white"
                        }`}
                      >
                        {isSaving ? (
                          <div className="flex items-center justify-center gap-2">
                            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                            Saving...
                          </div>
                        ) : (
                          "Save Profile"
                        )}
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </div>

            <OracleProfileCard />
            <GcpProfileCard />
            <AzureProfileCard />
          </div>

          <div className="bg-gray-800 rounded-lg p-6 mt-6 border border-red-900">
            <h2 className="text-xl font-semibold mb-2 text-red-400">
              Danger Zone
            </h2>
            <p className="text-gray-400 text-sm mb-4">
              Removes the daemon, all credentials, and WireGuard configs. The app will close automatically.
            </p>

            {uninstallError && (
              <div className="p-3 bg-red-900 border border-red-700 rounded-lg mb-4">
                <p className="text-red-300 text-sm">{uninstallError}</p>
              </div>
            )}

            {!isUninstallConfirming ? (
              <button
                onClick={() => setIsUninstallConfirming(true)}
                className="px-4 py-2 bg-red-700 hover:bg-red-600 text-white rounded-lg transition font-medium"
              >
                Uninstall App
              </button>
            ) : (
              <div className="flex items-center gap-3">
                <span className="text-gray-300 text-sm">Are you sure?</span>
                <button
                  onClick={() => setIsUninstallConfirming(false)}
                  className="px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition font-medium"
                >
                  Cancel
                </button>
                <button
                  onClick={handleUninstall}
                  disabled={isUninstalling}
                  className="px-4 py-2 bg-red-700 hover:bg-red-600 text-white rounded-lg transition font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isUninstalling ? "Uninstalling..." : "Yes, Uninstall"}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
