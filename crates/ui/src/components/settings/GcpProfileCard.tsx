import { useEffect, useRef, useState } from "react";
import { useCredentials, GcpCredentials } from "../../hooks";

interface GcpProfileCardProps {
  onCredentialsSaved: (provider: string) => void;
  onCredentialsDeleted: () => void;
  onProvisionRequested: (provider: string) => void;
  isProvisioned: boolean;
}

const GcpIcon = () => (
  <div className="w-12 h-12 bg-white/5 rounded-lg flex items-center justify-center flex-shrink-0 p-2">
    <img src="/cloud-providers/google-cloud-icon.svg" alt="GCP" className="w-full h-full object-contain" />
  </div>
);

export function GcpProfileCard({ onCredentialsSaved, onCredentialsDeleted, onProvisionRequested, isProvisioned }: GcpProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [hasCredentials, setHasCredentials] = useState<boolean | null>(null);
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const [projectId, setProjectId] = useState("");
  const [serviceAccountJson, setServiceAccountJson] = useState("");
  const [jsonAlreadySet, setJsonAlreadySet] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const {
    isSaving,
    error,
    successMessage,
    saveCredentials,
    deleteCredentials,
    loadCredentials,
    clearError,
  } = useCredentials();

  useEffect(() => {
    loadCredentials("gcp").then((existing) => {
      setHasCredentials(existing !== null);
    });
  }, []);

  const handleEditOpen = async () => {
    const existing = await loadCredentials("gcp");
    if (existing) {
      const gcpCredentials = existing as GcpCredentials;
      setProjectId(gcpCredentials.projectId);
      if (gcpCredentials.serviceAccountJson) {
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
      setHasCredentials(true);
      onCredentialsSaved("gcp");
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
        try {
          const parsed = JSON.parse(content);
          if (parsed.project_id && !projectId) {
            setProjectId(parsed.project_id);
          }
        } catch {
        }
      }
    };
    reader.readAsText(file);
    event.target.value = "";
  };

  const handleDeleteCredentials = async () => {
    const success = await deleteCredentials("gcp");
    if (success) {
      setHasCredentials(false);
      setIsConfirmingDelete(false);
      onCredentialsDeleted();
    }
  };

  const isFormValid =
    projectId.trim() && (serviceAccountJson.trim() || jsonAlreadySet);

  const showNotProvisionedWarning = hasCredentials === true && !isProvisioned;

  return (
    <div className={`rounded-lg p-6 mt-4 ${showNotProvisionedWarning ? "bg-gray-700 border-l-4 border-amber-500" : "bg-gray-700"}`}>
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <GcpIcon />
            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-lg text-white">Google Cloud Profile</h3>
                {hasCredentials && isProvisioned && (
                  <span className="text-xs px-2 py-0.5 bg-green-900 text-green-400 rounded-full border border-green-700">
                    Provisioned
                  </span>
                )}
                {showNotProvisionedWarning && (
                  <span className="text-xs px-2 py-0.5 bg-amber-900 text-amber-400 rounded-full border border-amber-700">
                    Not provisioned
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-400">
                GCP service-account credentials for Compute Engine deployment
              </p>
            </div>
          </div>
          {hasCredentials === null ? (
            <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
          ) : hasCredentials ? (
            <div className="flex items-center gap-2">
              {isConfirmingDelete ? (
                <>
                  <span className="text-sm text-gray-300">Delete?</span>
                  <button onClick={() => setIsConfirmingDelete(false)} className="px-3 py-1.5 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition text-sm">Cancel</button>
                  <button onClick={handleDeleteCredentials} className="px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded-lg transition text-sm">Confirm</button>
                </>
              ) : (
                <>
                  {isProvisioned ? (
                    <button onClick={() => onProvisionRequested("gcp")} className="p-2 text-gray-400 hover:text-blue-400 hover:bg-gray-600 rounded-lg transition" title="Re-provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </button>
                  ) : (
                    <button onClick={() => onProvisionRequested("gcp")} className="p-2 text-amber-400 hover:text-amber-300 hover:bg-gray-600 rounded-lg transition" title="Provision">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                      </svg>
                    </button>
                  )}
                  <button onClick={() => setIsConfirmingDelete(true)} className="p-2 text-gray-400 hover:text-red-400 hover:bg-gray-600 rounded-lg transition" title="Delete credentials">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  <button onClick={handleEditOpen} className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition font-medium flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Edit
                  </button>
                </>
              )}
            </div>
          ) : (
            <button onClick={handleEditOpen} className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition font-medium flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Add Provider
            </button>
          )}
        </div>
      ) : (
        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <GcpIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                {hasCredentials
                  ? "Edit Google Cloud Profile"
                  : "Add Google Cloud Profile"}
              </h3>
              <p className="text-sm text-gray-400">
                {hasCredentials
                  ? "Update your GCP service-account key"
                  : "Enter your GCP service-account key"}
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
