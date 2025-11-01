import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AwsRegion {
  name: string;
  country: string;
}

interface RegionGroup {
  continent: string;
  regions: AwsRegion[];
}

interface ExistingInstance {
  id: string;
  name?: string;
  state: string;
  public_ip_v4: string;
  public_ip_v6: string;
  region?: string;
}

interface ServerDetails {
  instance_id: string;
  public_ip_v4: string;
  public_ip_v6?: string;
  region: string;
  client_private_key: string;
  server_public_key: string;
}

export function VpnPage() {
  const [regions, setRegions] = useState<AwsRegion[]>([]);
  const [groupedRegions, setGroupedRegions] = useState<RegionGroup[]>([]);
  const [selectedRegion, setSelectedRegion] = useState<AwsRegion | null>(null);
  const [isSpawning, setIsSpawning] = useState(false);
  const [isTerminating, setIsTerminating] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [showSettingsModal, setShowSettingsModal] = useState(false);
  const [newAccessKey, setNewAccessKey] = useState("");
  const [newSecretKey, setNewSecretKey] = useState("");
  const [isSavingCredentials, setIsSavingCredentials] = useState(false);
  const [serverStatus, setServerStatus] = useState<
    | "idle"
    | "spawning"
    | "running"
    | "error"
    | "terminating"
    | "connecting"
    | "connected"
  >("idle");
  const [selectedInstance, setSelectedInstance] =
    useState<ServerDetails | null>(null);
  const [existingInstances, setExistingInstances] = useState<
    ExistingInstance[]
  >([]);

  // Load regions and existing instances on component mount
  useEffect(() => {
    loadRegions();
  }, []);

  // Group regions by continent when regions change
  useEffect(() => {
    if (regions.length > 0) {
      const grouped = groupRegionsByContinent(regions);
      setGroupedRegions(grouped);
    }
  }, [regions]);

  // Load instances when regions are available
  useEffect(() => {
    if (regions.length > 0) {
      loadExistingInstances();
    }
  }, [regions]);

  const groupRegionsByContinent = (regions: AwsRegion[]): RegionGroup[] => {
    const continentMap: Record<string, string> = {
      // North America
      us: "North America",
      ca: "North America",

      // Europe
      eu: "Europe",

      // Asia Pacific
      ap: "Asia Pacific",

      // South America
      sa: "South America",

      // Middle East
      me: "Middle East",

      // Africa
      af: "Africa",
    };

    const groups: Record<string, AwsRegion[]> = {};

    regions.forEach((region) => {
      const prefix = region.name.split("-")[0];
      const continent = continentMap[prefix] || "Other";

      if (!groups[continent]) {
        groups[continent] = [];
      }
      groups[continent].push(region);
    });

    // Convert to array and sort
    return Object.entries(groups)
      .map(([continent, regions]) => ({
        continent,
        regions: regions.sort((a, b) => a.name.localeCompare(b.name)),
      }))
      .sort((a, b) => a.continent.localeCompare(b.continent));
  };

  const loadRegions = async () => {
    try {
      console.log("Loading regions...");
      const fetchedRegions = (await invoke("get_regions")) as AwsRegion[];
      console.log({ fetchedRegions });
      setRegions(fetchedRegions);
    } catch (error) {
      console.error("Failed to load regions:", error);
    }
  };

  const loadExistingInstances = async () => {
    setIsLoading(true);

    // Wait for regions to be loaded if they're not already
    if (regions.length === 0) {
      return;
    }

    // Check all regions in parallel for existing instances
    const regionPromises = regions.map(async (region) => {
      try {
        const instances = (await invoke("list_instances", {
          region: region.name,
        })) as ExistingInstance[];

        // Add region info to each instance
        return instances.map((instance) => ({
          ...instance,
          region: region.name,
        }));
      } catch (error) {
        console.warn(`Failed to load instances from ${region.name}:`, error);
        return []; // Return empty array on error
      }
    });

    // Wait for all regions to be checked
    const allRegionResults = await Promise.all(regionPromises);

    // Flatten the results into a single array
    const allInstances = allRegionResults.flat();

    setExistingInstances(allInstances);

    // If we have a running instance, set it as the current server
    const runningInstance = allInstances.find(
      (instance) => instance.state === "running"
    );
    if (runningInstance) {
      setSelectedInstance({
        instance_id: runningInstance.id,
        public_ip_v4: runningInstance.public_ip_v4,
        public_ip_v6: runningInstance.public_ip_v6,
        region: runningInstance.region || "",
        client_private_key: "", // We don't have this from the API
        server_public_key: "", // We don't have this from the API
      });
      setServerStatus("running");

      // Set the selected region to match the running instance
      const regionInfo = regions.find((r) => r.name === runningInstance.region);
      if (regionInfo) {
        setSelectedRegion(regionInfo);
      }
    }

    setIsLoading(false);
  };

  const handleRegionSelect = (region: AwsRegion) => {
    setSelectedRegion(region);
    // Clear selected instance when selecting a region (to show spawn controls)
    setSelectedInstance(null);
    setServerStatus("idle");
  };

  const handleInstanceSelect = (instance: ExistingInstance) => {
    // Convert ExistingInstance to ServerDetails format
    const instanceDetails: ServerDetails = {
      instance_id: instance.id,
      public_ip_v4: instance.public_ip_v4,
      public_ip_v6: instance.public_ip_v6,
      region: instance.region || "",
      client_private_key: "", // We don't have this from the API
      server_public_key: "", // We don't have this from the API
    };

    setSelectedInstance(instanceDetails);
    setServerStatus("running");

    // Set the selected region to match the instance
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

      setExistingInstances((prev) => [...prev, newInstance]);
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
      setSelectedInstance(null);
      setServerStatus("idle");

      // Remove the terminated instance from local state instead of refetching
      setExistingInstances((prev) =>
        prev.filter((instance) => instance.id !== selectedInstance.instance_id)
      );
    } catch (error) {
      console.error("Failed to terminate server:", error);
      setServerStatus("error");
    } finally {
      setIsTerminating(false);
    }
  };

  const handleConnectToVpn = async () => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setServerStatus("connecting");

    try {
      const response = await invoke("connect", {
        instanceId: selectedInstance.instance_id,
        region: selectedInstance.region,
      });

      console.log("VPN connected:", response);
      setServerStatus("connected");
    } catch (error) {
      console.error("Failed to connect to VPN:", error);
      setServerStatus("error");
      alert(`Failed to connect to VPN: ${error}`);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnectFromVpn = async () => {
    setServerStatus("idle");

    try {
      const response = await invoke("disconnect");
      console.log("VPN disconnected:", response);
    } catch (error) {
      console.error("Failed to disconnect from VPN:", error);
      alert(`Failed to disconnect from VPN: ${error}`);
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

      // Reload instances with new credentials
      await loadExistingInstances();

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

          {/* Settings Button */}
          <button
            onClick={() => setShowSettingsModal(true)}
            className="p-2 rounded-lg bg-gray-700 hover:bg-gray-600 transition-colors"
            title="Update AWS Credentials"
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
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </button>
        </div>
      </div>

      <div className="flex-1 flex min-h-0">
        {/* Show loading state */}
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <div className="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
              <p className="text-gray-300">Loading existing instances...</p>
            </div>
          </div>
        ) : (
          <>
            {/* Existing Instances Section */}
            {existingInstances.length > 0 && (
              <div className="flex-1 p-6 flex flex-col min-h-0">
                <div className="flex-1 flex flex-col min-h-0">
                  <h2 className="text-xl font-semibold mb-4 text-blue-400">
                    Existing VPN Servers
                  </h2>
                  <div className="flex-1 overflow-y-auto min-h-0">
                    <div className="grid grid-cols-1 gap-6 p-2">
                      {existingInstances.map((instance) => (
                        <div
                          key={instance.id}
                          onClick={() => handleInstanceSelect(instance)}
                          className={`p-4 rounded-lg cursor-pointer transition-all border ${
                            selectedInstance?.instance_id === instance.id
                              ? "bg-blue-600 border-blue-500 text-white transform scale-102 shadow-lg"
                              : "bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600"
                          }`}
                          style={{
                            transformOrigin: "center",
                          }}
                        >
                          <div className="flex items-center justify-between mb-2">
                            <h3 className="font-semibold text-lg">
                              {instance.name || "VPN Server"}
                            </h3>
                            <div className="flex items-center gap-2">
                              <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                              <span className="text-sm text-green-400">
                                Running
                              </span>
                              {selectedInstance?.instance_id ===
                                instance.id && (
                                <div className="w-3 h-3 bg-white rounded-full ml-2"></div>
                              )}
                            </div>
                          </div>
                          <p className="text-sm text-gray-400 mb-1">
                            <span className="font-medium">Instance ID:</span>{" "}
                            {instance.id}
                          </p>
                          <p className="text-sm text-gray-400 mb-1">
                            <span className="font-medium">Public IP:</span>{" "}
                            {instance.public_ip_v4}
                          </p>
                          <p className="text-sm text-gray-400">
                            <span className="font-medium">Region:</span>{" "}
                            {instance.region}
                          </p>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Region List */}
            <div className="flex-1 p-6 flex flex-col min-h-0">
              <div className="flex-1 flex flex-col min-h-0">
                <h2 className="text-xl font-semibold mb-4 text-blue-400">
                  {existingInstances.length > 0
                    ? "Deploy in New Region"
                    : "Available AWS Regions"}
                </h2>
                <div className="flex-1 overflow-y-auto min-h-0">
                  <div className="space-y-8 p-2">
                    {groupedRegions.map((group) => (
                      <div key={group.continent} className="space-y-4">
                        <h3 className="text-lg font-semibold text-blue-300 border-b border-gray-700 pb-2">
                          {group.continent}
                        </h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                          {group.regions.map((region) => (
                            <div
                              key={region.name}
                              onClick={() => handleRegionSelect(region)}
                              className={`p-4 rounded-lg cursor-pointer transition-all border ${
                                selectedRegion?.name === region.name
                                  ? "bg-blue-600 border-blue-500 text-white transform scale-102 shadow-lg"
                                  : "bg-gray-800 border-gray-700 hover:bg-gray-700 hover:border-gray-600 text-gray-300"
                              }`}
                              style={{
                                transformOrigin: "center",
                              }}
                            >
                              <div className="flex items-center justify-between mb-2">
                                <h4 className="font-semibold text-base">
                                  {region.name}
                                </h4>
                                {selectedRegion?.name === region.name && (
                                  <div className="w-3 h-3 bg-white rounded-full"></div>
                                )}
                              </div>
                              <p className="text-sm opacity-75 mb-1">
                                {region.country}
                              </p>
                              <p className="text-xs opacity-60 font-mono">
                                {region.name}
                              </p>
                            </div>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
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
                    onClick={handleConnectToVpn}
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
