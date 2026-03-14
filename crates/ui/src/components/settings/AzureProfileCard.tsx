import { useState } from "react";
import { useCredentials, AzureCredentials } from "../../hooks";

interface AzureProfileCardProps {

  onSaveSuccess?: () => void;
}

const AzureIcon = () => (
  <div className="w-12 h-12 bg-sky-600 rounded-lg flex items-center justify-center flex-shrink-0">
    <span className="text-white font-bold text-xs tracking-wider">AZ</span>
  </div>
);

export function AzureProfileCard({ onSaveSuccess }: AzureProfileCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [subscriptionId, setSubscriptionId] = useState("");
  const [tenantId, setTenantId] = useState("");
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [secretAlreadySet, setSecretAlreadySet] = useState(false);

  const {
    isSaving,
    error,
    successMessage,
    saveCredentials,
    loadCredentials,
    clearError,
  } = useCredentials();

  const handleEditOpen = async () => {
    const existing = await loadCredentials("azure");
    if (existing) {
      const azure = existing as AzureCredentials;
      setSubscriptionId(azure.subscriptionId);
      setTenantId(azure.tenantId);
      setClientId(azure.clientId);
      if (azure.clientSecret) {
        setSecretAlreadySet(true);
      }
    }
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    setSubscriptionId("");
    setTenantId("");
    setClientId("");
    setClientSecret("");
    setSecretAlreadySet(false);
    clearError();
  };

  const handleSave = async () => {
    const success = await saveCredentials("azure", {
      subscriptionId: subscriptionId.trim(),
      tenantId: tenantId.trim(),
      clientId: clientId.trim(),
      clientSecret: clientSecret.trim(),
    });

    if (success) {
      setSubscriptionId("");
      setTenantId("");
      setClientId("");
      setClientSecret("");
      setSecretAlreadySet(false);
      setIsEditing(false);
      onSaveSuccess?.();
    }
  };

  const isFormValid =
    subscriptionId.trim() &&
    tenantId.trim() &&
    clientId.trim() &&
    (clientSecret.trim() || secretAlreadySet);

  return (
    <div className="bg-gray-700 rounded-lg p-6 mt-4">
      {!isEditing ? (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <AzureIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                Azure Profile
              </h3>
              <p className="text-sm text-gray-400">
                Azure service-principal credentials for VM deployment
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
            <AzureIcon />
            <div>
              <h3 className="font-semibold text-lg text-white">
                Edit Azure Profile
              </h3>
              <p className="text-sm text-gray-400">
                Update your Azure service-principal credentials
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Subscription ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
              </p>
              <input
                type="text"
                value={subscriptionId}
                onChange={(e) => setSubscriptionId(e.target.value)}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Tenant ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                Your Azure Active Directory tenant ID (UUID)
              </p>
              <input
                type="text"
                value={tenantId}
                onChange={(e) => setTenantId(e.target.value)}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Client ID
              </label>
              <p className="text-xs text-gray-500 mb-2">
                Application (client) ID of the service principal
              </p>
              <input
                type="text"
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-sm"
                placeholder="00000000-0000-0000-0000-000000000000"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Client Secret
              </label>
              {secretAlreadySet && !clientSecret && (
                <p className="text-xs text-green-400 mb-2">
                  ✓ Client secret already configured — enter a new value to
                  replace it
                </p>
              )}
              {!secretAlreadySet && !clientSecret && (
                <p className="text-xs text-gray-500 mb-2">
                  Secret value from your app registration
                </p>
              )}
              <input
                type="password"
                value={clientSecret}
                onChange={(e) => setClientSecret(e.target.value)}
                className="w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-lg text-white focus:border-blue-500 focus:outline-none font-mono text-sm"
                placeholder={
                  secretAlreadySet ? "Enter new secret to replace" : ""
                }
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
