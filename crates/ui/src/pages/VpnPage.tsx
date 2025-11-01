import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRegions, useInstances, useVpnConnection } from "../hooks";
import { AwsRegion, ExistingInstance, ServerDetails } from "../types";
import { LoadingSpinner } from "../components/common/LoadingSpinner";
import { SettingsButton } from "../components/settings/SettingsButton";
import { InstanceList } from "../components/instances/InstanceList";
import { RegionList } from "../components/regions/RegionList";

export function VpnPage() {
  // Hooks for state management
  const {
    regions,
    groupedRegions,
    selectedRegion,
    isLoading: regionsLoading,
    handleRegionSelect,
  } = useRegions();

  const {
    existingInstances,
    selectedInstance,
    isLoading: instancesLoading,
    handleInstanceSelect,
    addInstance,
    removeInstance,
    clearSelectedInstance,
    setSelectedInstance,
  } = useInstances(regions);

  const {
    serverStatus,
    isConnecting,
    setServerStatus,
    handleConnectToVpn,
    handleDisconnectFromVpn,
  } = useVpnConnection();

  // Local state for server operations
  const [isSpawning, setIsSpawning] = useState(false);
  const [isTerminating, setIsTerminating] = useState(false);

  // Settings modal state
  const [showSettingsModal, setShowSettingsModal] = useState(false);
  const [newAccessKey, setNewAccessKey] = useState("");
  const [newSecretKey, setNewSecretKey] = useState("");
  const [isSavingCredentials, setIsSavingCredentials] = useState(false);

  const handleRegionSelectWrapper = (region: AwsRegion) => {
    handleRegionSelect(region);
    // Clear selected instance when selecting a region (to show spawn controls)
    clearSelectedInstance();
    setServerStatus("idle");
  };

  const handleInstanceSelectWrapper = (instance: ExistingInstance) => {
    handleInstanceSelect(instance);
    setServerStatus("running");
  };

  const handleSpawnServer = async () => {
    if (!selectedRegion) return;

    setIsSpawning(true);
    setServerStatus("spawning");

    try {
      // Call your Tauri command to spawn EC2 instance
      const result = await invoke("spawn_instance", {
        region: selectedRegion.name,
      });

      console.log("Server spawned:", result);
      const serverDetails = result as ServerDetails;
      setSelectedInstance(serverDetails);
      setServerStatus("running");

      // Add the new instance to local state instead of refetching
      const newInstance: ExistingInstance = {
        id: serverDetails.instance_id,
        name: "VPN Server",
        state: "running",
        public_ip_v4: serverDetails.public_ip_v4,
        public_ip_v6: serverDetails.public_ip_v6 || "",
        region: selectedRegion.name,
      };

      addInstance(newInstance);
    } catch (error) {
      console.error("Failed to spawn server:", error);
      setServerStatus("error");
    } finally {
      setIsSpawning(false);
    }
  };

  const handleTerminateServer = async () => {
    if (!selectedInstance) return;

    setIsTerminating(true);
    setServerStatus("terminating");

    try {
      // Call the Tauri command to terminate the instance
      const result = await invoke("terminate_instance", {
        instanceId: selectedInstance.instance_id,
        region: selectedInstance.region,
      });

      console.log("Server terminated:", result);
      clearSelectedInstance();
      setServerStatus("idle");

      // Remove the terminated instance from local state instead of refetching
      removeInstance(selectedInstance.instance_id);
    } catch (error) {
      console.error("Failed to terminate server:", error);
      setServerStatus("error");
    } finally {
      setIsTerminating(false);
    }
  };

  const handleUpdateCredentials = async () => {
    if (!newAccessKey.trim() || !newSecretKey.trim()) {
      alert("Please fill in both access key and secret key");
      return;
    }

    setIsSavingCredentials(true);

    try {
      await invoke("save_credentials", {
        cloudProviderName: "aws",
        accessKeyId: newAccessKey.trim(),
        secretAccessKey: newSecretKey.trim(),
      });

      // Clear form and close modal
      setNewAccessKey("");
      setNewSecretKey("");
      setShowSettingsModal(false);

      alert("Credentials updated successfully!");
    } catch (error) {
      console.error("Failed to update credentials:", error);
      alert(
        "Failed to update credentials. Please check your keys and try again."
      );
    } finally {
      setIsSavingCredentials(false);
    }
  };

  const isLoading = regionsLoading || instancesLoading;

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white overflow-hidden">
      {/* Header */}
      <div className="bg-gray-800 p-6 border-b border-gray-700 flex-shrink-0">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold mb-2 text-blue-400">
              VPN Server Deployment
            </h1>
            <p className="text-gray-300">
              Select an AWS region to deploy your VPN server
            </p>
          </div>
          <SettingsButton onClick={() => setShowSettingsModal(true)} />
        </div>
      </div>

      <div className="flex-1 flex min-h-0">
        {/* Show loading state */}
        {isLoading ? (
          <LoadingSpinner message="Loading existing instances..." />
        ) : (
          <>
            {/* Existing Instances Section */}
            <InstanceList
              instances={existingInstances}
              selectedInstance={selectedInstance}
              onInstanceSelect={handleInstanceSelectWrapper}
            />

            {/* Region List */}
            <RegionList
              groupedRegions={groupedRegions}
              selectedRegion={selectedRegion}
              onRegionSelect={handleRegionSelectWrapper}
              existingInstancesCount={existingInstances.length}
            />
          </>
        )}

        {/* Control Panel */}
        <div className="w-80 bg-gray-800 p-6 border-l border-gray-700 flex flex-col min-h-0">
          <h2 className="text-xl font-semibold mb-4 text-blue-400">
            {selectedInstance ? "Instance Details" : "Server Control"}
          </h2>

          <div className="flex-1 overflow-y-auto">
            {selectedInstance ? (
              /* Instance Details View */
              <>
                <div className="mb-6">
                  <h3 className="text-lg font-medium mb-2">
                    Selected Instance
                  </h3>
                  <div className="bg-gray-700 rounded-lg p-4">
                    <p className="font-medium text-blue-300 mb-2">
                      {existingInstances.find(
                        (i) => i.id === selectedInstance.instance_id
                      )?.name || "VPN Server"}
                    </p>
                    <p className="text-sm text-gray-400 mb-1">
                      <span className="font-medium">Instance ID:</span>{" "}
                      {selectedInstance.instance_id}
                    </p>
                    <p className="text-sm text-gray-400 mb-1">
                      <span className="font-medium">Public IP:</span>{" "}
                      {selectedInstance.public_ip_v4}
                    </p>
                    <p className="text-sm text-gray-400">
                      <span className="font-medium">Region:</span>{" "}
                      {selectedInstance.region}
                    </p>
                  </div>
                </div>

                {/* Instance Actions */}
                <div className="space-y-3">
                  <button
                    onClick={() => handleConnectToVpn(selectedInstance)}
                    disabled={isConnecting || serverStatus === "connected"}
                    className={`w-full px-4 py-3 rounded-lg transition font-medium shadow-lg hover:shadow-xl ${
                      isConnecting
                        ? "bg-gray-600 text-gray-400 cursor-not-allowed"
                        : serverStatus === "connected"
                          ? "bg-green-800 text-green-200 cursor-not-allowed"
                          : "bg-green-600 hover:bg-green-700 text-white"
                    }`}
                  >
                    {isConnecting ? (
                      <div className="flex items-center justify-center gap-2">
                        <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        Connecting...
                      </div>
                    ) : serverStatus === "connected" ? (
                      "Connected to VPN"
                    ) : (
                      "Connect to VPN"
                    )}
                  </button>

                  {serverStatus === "connected" && (
                    <button
                      onClick={handleDisconnectFromVpn}
                      className="w-full px-4 py-3 bg-yellow-600 hover:bg-yellow-700 text-white rounded-lg transition font-medium shadow-lg hover:shadow-xl"
                    >
                      Disconnect from VPN
                    </button>
                  )}

                  {serverStatus !== "connected" && (
                    <button
                      onClick={handleTerminateServer}
                      disabled={isTerminating}
                      className={`w-full px-4 py-3 rounded-lg transition font-medium shadow-lg hover:shadow-xl ${
                        isTerminating
                          ? "bg-gray-600 text-gray-400 cursor-not-allowed"
                          : "bg-red-600 hover:bg-red-700 text-white"
                      }`}
                    >
                      {isTerminating ? (
                        <div className="flex items-center justify-center gap-2">
                          <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                          Terminating...
                        </div>
                      ) : (
                        "Terminate Server"
                      )}
                    </button>
                  )}
                </div>
              </>
            ) : (
              /* Region Selection / Spawn Controls View */
              <>
                {selectedRegion ? (
                  <div className="mb-6">
                    <h3 className="text-lg font-medium mb-2">
                      Selected Region
                    </h3>
                    <div className="bg-gray-700 rounded-lg p-4">
                      <p className="font-medium text-blue-300">
                        {selectedRegion.name}
                      </p>
                      <p className="text-sm text-gray-400 mt-1">
                        {selectedRegion.country}
                      </p>
                    </div>
                  </div>
                ) : (
                  <div className="mb-6">
                    <p className="text-gray-400">
                      Select a region from the list to continue
                    </p>
                  </div>
                )}

                {/* Server Status */}
                <div className="mb-6">
                  <h3 className="text-lg font-medium mb-2">Server Status</h3>
                  <div className="flex items-center gap-3 p-3 bg-gray-700 rounded-lg">
                    <div
                      className={`w-4 h-4 rounded-full ${
                        serverStatus === "running"
                          ? "bg-green-500"
                          : serverStatus === "spawning"
                            ? "bg-yellow-500 animate-pulse"
                            : serverStatus === "terminating"
                              ? "bg-orange-500 animate-pulse"
                              : serverStatus === "error"
                                ? "bg-red-500"
                                : "bg-gray-500"
                      }`}
                    />
                    <span className="capitalize">
                      {serverStatus === "idle"
                        ? "Ready to deploy"
                        : serverStatus}
                    </span>
                  </div>
                </div>

                {/* Spawn Actions */}
                <div className="space-y-3">
                  <button
                    onClick={handleSpawnServer}
                    disabled={!selectedRegion || isSpawning || isTerminating}
                    className={`w-full px-4 py-3 rounded-lg transition font-medium ${
                      !selectedRegion || isSpawning || isTerminating
                        ? "bg-gray-600 text-gray-400 cursor-not-allowed"
                        : "bg-blue-600 hover:bg-blue-700 text-white shadow-lg hover:shadow-xl"
                    }`}
                  >
                    {isSpawning ? (
                      <div className="flex items-center justify-center gap-2">
                        <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        Deploying Server...
                      </div>
                    ) : (
                      "Deploy VPN Server"
                    )}
                  </button>

                  {serverStatus === "error" && (
                    <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                      <p className="text-red-300 text-sm">
                        Failed to deploy server. Please try again or check your
                        credentials.
                      </p>
                    </div>
                  )}
                </div>
              </>
            )}

            {/* Info */}
            <div className="mt-6 p-3 bg-gray-700 rounded-lg">
              <h4 className="font-medium mb-2 text-sm">ℹ️ Information</h4>
              <p className="text-xs text-gray-400 leading-relaxed">
                Deploying a server will create an EC2 instance in your selected
                region. You'll be charged according to AWS pricing for the
                instance runtime.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Settings Modal */}
      {showSettingsModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-blue-400">
                Update AWS Credentials
              </h2>
              <button
                onClick={() => setShowSettingsModal(false)}
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
                  value={newAccessKey}
                  onChange={(e) => setNewAccessKey(e.target.value)}
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
                  value={newSecretKey}
                  onChange={(e) => setNewSecretKey(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                  placeholder="Enter your secret key..."
                />
              </div>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => setShowSettingsModal(false)}
                  className="flex-1 px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg transition"
                >
                  Cancel
                </button>
                <button
                  onClick={handleUpdateCredentials}
                  disabled={
                    isSavingCredentials ||
                    !newAccessKey.trim() ||
                    !newSecretKey.trim()
                  }
                  className={`flex-1 px-4 py-2 rounded-lg transition ${
                    isSavingCredentials ||
                    !newAccessKey.trim() ||
                    !newSecretKey.trim()
                      ? "bg-gray-600 text-gray-400 cursor-not-allowed"
                      : "bg-blue-600 hover:bg-blue-700 text-white"
                  }`}
                >
                  {isSavingCredentials ? (
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
      )}
    </div>
  );
}
