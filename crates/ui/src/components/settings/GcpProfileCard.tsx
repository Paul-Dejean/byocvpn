import { useRef, useState } from "react";
import { useCredentials, GcpCredentials } from "../../hooks";

/**
 * Props for the GcpProfileCard component
 */
interface GcpProfileCardProps {
  /** Optional callback invoked after credentials are saved successfully */
  onSaveSuccess?: () => void;
}

const GcpIcon = () => (
  <div className="w-12 h-12 bg-blue-600 rounded-lg flex items-center justify-center flex-shrink-0">
    <span className="text-white font-bold text-xs tracking-wider">GCP</span>
  </div>
);

/**
 * Settings card for managing GCP service-account credentials.
 * Supports pasting the JSON key directly or loading it from a .json file.
 */
export function GcpProfileCard({ onSaveSuccess }: GcpProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [projectId, setProjectId] = useState("");
  const [serviceAccountJson, setServiceAccountJson] = useState("");
  const [jsonAlreadySet, setJsonAlreadySet] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const {
    isSaving,
    error,
    successMessage,
    saveCredentials,
    loadCredentials,
    clearError,
  } = useCredentials();

  const handleEditOpen = async () => {
    const existing = await loadCredentials("gcp");
    if (existing) {
      const g = existing as GcpCredentials;
      setProjectId(g.projectId);
      if (g.serviceAccountJson) {
        setJsonAlreadySet(true);
      }
    }
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    setProjectId("");
    setServiceAccountJson("");
    setJsonAlreadySet(false);
    clearError();
  };

  const handleSave = async () => {
    const success = await saveCredentials("gcp", {
      projectId: projectId.trim(),
      serviceAccountJson: serviceAccountJson.trim(),
    });

    if (success) {
      setProjectId("");
      setServiceAccountJson("");
      setJsonAlreadySet(false);
      setIsEditing(false);
      onSaveSuccess?.();
    }
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const content = loadEvent.target?.result;
      if (typeof content === "string") {
        setServiceAccountJson(content);
        // Auto-fill project_id from the JSON if present.
        try {
          const parsed = JSON.parse(content);
          if (parsed.project_id && !projectId) {
            setProjectId(parsed.project_id);
          }
        } catch {
          // Not valid JSON yet — user may still be editing.
        }
      }
    };
    reader.readAsText(file);
    event.target.value = "";
  };

  const isFormValid =
    projectId.trim() && (serviceAccountJson.trim() || jsonAlreadySet);

  return (
    <div className="bg-gray-700 rounded-lg p-6 mt-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <GcpIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                Google Cloud Profile
              </h3>
              <p className="text-sm text-gray-400">
                GCP service-account credentials for Compute Engine deployment
              </p>
            </div>
          </div>
          <button
            onClick={handleEditOpen}
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
          <div className="flex items-center gap-4">
            <GcpIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                Edit Google Cloud Profile
              </h3>
              <p className="text-sm text-gray-400">
                Update your GCP service-account key
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Project ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                e.g. my-project-123456
              </p>
              <input
                type="text"
                value={projectId}
                onChange={(e) => setProjectId(e.target.value)}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-sm"
                placeholder="my-gcp-project"
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="block text-sm font-medium text-gray-300">
                  Service Account Key (.json)
                </label>
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  className="text-xs px-3 py-1 bg-gray-600 hover:bg-gray-500 text-gray-300 rounded-lg transition flex items-center gap-1.5"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-3.5 w-3.5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
                    />
                  </svg>
                  Load from file
                </button>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".json,application/json"
                  onChange={handleFileChange}
                  className="hidden"
                />
              </div>
              {jsonAlreadySet && !serviceAccountJson && (
                <p className="text-xs text-green-400 mb-2">
                  ✓ Service account key already configured — load a new file or
                  paste below to replace it
                </p>
              )}
              {!jsonAlreadySet && !serviceAccountJson && (
                <p className="text-xs text-gray-500 mb-2">
                  Paste the contents of your service-account JSON key file or
                  use "Load from file"
                </p>
              )}
              <textarea
                value={serviceAccountJson}
                onChange={(e) => setServiceAccountJson(e.target.value)}
                rows={6}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-xs resize-none"
                placeholder='{"type":"service_account","project_id":"..."}'
              />
            </div>

            {error && (
              <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                <p className="text-red-300 text-sm">{error}</p>
              </div>
            )}

            {successMessage && (
              <div className="p-3 bg-green-900 border border-green-700 rounded-lg">
                <p className="text-green-300 text-sm">{successMessage}</p>
              </div>
            )}

            <div className="flex gap-3 pt-4">
              <button
                onClick={handleCancel}
                className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition font-medium"
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                disabled={isSaving || !isFormValid}
                className={`flex-1 px-4 py-2 rounded-lg transition font-medium ${
                  isSaving || !isFormValid
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
  );
}
