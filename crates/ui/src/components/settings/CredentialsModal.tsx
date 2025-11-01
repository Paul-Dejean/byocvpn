import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CredentialsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export function CredentialsModal({
  isOpen,
  onClose,
  onSuccess,
}: CredentialsModalProps) {
  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    if (!accessKey.trim() || !secretKey.trim()) {
      alert("Please fill in both access key and secret key");
      return;
    }

    setIsSaving(true);

    try {
      await invoke("save_credentials", {
        cloudProviderName: "aws",
        accessKeyId: accessKey.trim(),
        secretAccessKey: secretKey.trim(),
      });

      // Clear form and close modal
      setAccessKey("");
      setSecretKey("");
      onClose();
      onSuccess?.();

      alert("Credentials updated successfully!");
    } catch (error) {
      console.error("Failed to update credentials:", error);
      alert(
        "Failed to update credentials. Please check your keys and try again."
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    setAccessKey("");
    setSecretKey("");
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-blue-400">
            Update AWS Credentials
          </h2>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-white"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
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
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:border-blue-500 focus:outline-none"
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
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:border-blue-500 focus:outline-none"
              placeholder="Enter your secret key..."
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              onClick={handleClose}
              className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving || !accessKey.trim() || !secretKey.trim()}
              className={`flex-1 px-4 py-2 rounded-lg transition ${
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
                "Update Credentials"
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
