import { useState } from "react";
import { useCredentials } from "../hooks/useCredentials";

interface SettingsPageProps {
  onNavigateBack?: () => void;
}

export function SettingsPage({ onNavigateBack }: SettingsPageProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");

  const { isSaving, error, successMessage, saveCredentials } = useCredentials();

  const handleEditProfile = () => {
    setIsEditing(true);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setAccessKey("");
    setSecretKey("");
  };

  const handleSaveProfile = async () => {
    if (!accessKey.trim() || !secretKey.trim()) {
      return;
    }

    const success = await saveCredentials("aws", accessKey, secretKey);

    if (success) {
      // Clear form and exit edit mode
      setAccessKey("");
      setSecretKey("");
      setIsEditing(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
      {/* Header */}
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

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto">
          {/* Manage Profiles Section */}
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-6 text-blue-400">
              Manage Profiles
            </h2>

            {/* AWS Profile Row */}
            <div className="bg-gray-700 rounded-lg p-6">
              {!isEditing ? (
                /* Profile Display Mode */
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
                /* Profile Edit Mode */
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
                      <label className="block text-sm font-medium text-gray-300 mb-2">
                        Access Key ID
                      </label>
                      <input
                        type="text"
                        value={accessKey}
                        onChange={(e) => setAccessKey(e.target.value)}
                        className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                        placeholder="AKIA..."
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-300 mb-2">
                        Secret Access Key
                      </label>
                      <input
                        type="password"
                        value={secretKey}
                        onChange={(e) => setSecretKey(e.target.value)}
                        className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                        placeholder="Enter your secret key..."
                      />
                    </div>

                    {/* Error Display */}
                    {error && (
                      <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                        <p className="text-red-300 text-sm">{error}</p>
                      </div>
                    )}

                    {/* Success Display */}
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
                        disabled={
                          isSaving || !accessKey.trim() || !secretKey.trim()
                        }
                        className={`flex-1 px-4 py-2 rounded-lg transition font-medium ${
                          isSaving || !accessKey.trim() || !secretKey.trim()
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
          </div>
        </div>
      </div>
    </div>
  );
}
