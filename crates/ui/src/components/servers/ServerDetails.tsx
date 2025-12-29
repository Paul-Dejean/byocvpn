import { ExistingInstance, RegionGroup } from "../../types";

interface ServerDetailsProps {
  instance: ExistingInstance;
  groupedRegions: RegionGroup[];
  isConnecting: boolean;
  isTerminating: boolean;
  vpnError: string | null;
  terminateError: string | null;
  onConnect: (data: {
    instance_id: string;
    public_ip_v4: string;
    public_ip_v6: string | undefined;
    region: string | undefined;
    client_private_key: string;
    server_public_key: string;
  }) => void;
  onTerminate: (instance: ExistingInstance) => void;
}

export function ServerDetails({
  instance,
  groupedRegions,
  isConnecting,
  isTerminating,
  vpnError,
  terminateError,
  onConnect,
  onTerminate,
}: ServerDetailsProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === regionName) as any;
    return region?.flag || "🌍";
  };

  return (
    <div className="flex-1 flex flex-col bg-gray-900">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {/* Instance Details Card */}
          <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <div className="flex items-center gap-3 mb-4">
              <span className="text-3xl">{getRegionFlag(instance.region)}</span>
              <div>
                <h3 className="text-lg font-semibold text-blue-400">
                  Server Details
                </h3>
                <p className="text-sm text-gray-400">{instance.region}</p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <p className="text-xs text-gray-400 mb-1">Instance ID</p>
                <p className="text-sm font-mono text-white">{instance.id}</p>
              </div>
              <div>
                <p className="text-xs text-gray-400 mb-1">IPv4 Address</p>
                <p className="text-sm font-mono text-white">
                  {instance.public_ip_v4}
                </p>
              </div>
              {instance.public_ip_v6 && (
                <div>
                  <p className="text-xs text-gray-400 mb-1">IPv6 Address</p>
                  <p className="text-sm font-mono text-white">
                    {instance.public_ip_v6}
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Actions */}
          <div className="space-y-3">
            <button
              onClick={() =>
                onConnect({
                  instance_id: instance.id,
                  public_ip_v4: instance.public_ip_v4,
                  public_ip_v6: instance.public_ip_v6,
                  region: instance.region,
                  client_private_key: "",
                  server_public_key: "",
                })
              }
              disabled={isConnecting}
              className="w-full px-6 py-4 bg-green-600 hover:bg-green-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isConnecting ? (
                <div className="flex items-center justify-center gap-2">
                  <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                  Connecting...
                </div>
              ) : (
                "Connect to VPN"
              )}
            </button>

            <button
              onClick={() => {
                onTerminate(instance);
              }}
              disabled={isTerminating}
              className="w-full px-6 py-4 bg-red-600 hover:bg-red-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isTerminating ? (
                <div className="flex items-center justify-center gap-2">
                  <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                  Terminating...
                </div>
              ) : (
                "Terminate Server"
              )}
            </button>
          </div>

          {/* Errors */}
          {vpnError && (
            <div className="p-4 bg-red-900/20 border border-red-500/50 rounded-lg">
              <p className="text-red-300 text-sm">{vpnError}</p>
            </div>
          )}
          {terminateError && (
            <div className="p-4 bg-red-900/20 border border-red-500/50 rounded-lg">
              <p className="text-red-300 text-sm">{terminateError}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
