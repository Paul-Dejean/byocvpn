import {
  Instance,
  RegionGroup,
  SpawnJobState,
  SpawnStepStatus,
} from "../../types";

function StepIndicator({ status }: { status: SpawnStepStatus }) {
  if (status === "running") {
    return (
      <div className="w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full animate-spin flex-shrink-0" />
    );
  }
  if (status === "completed") {
    return (
      <svg
        className="w-5 h-5 text-green-400 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fillRule="evenodd"
          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
          clipRule="evenodd"
        />
      </svg>
    );
  }
  if (status === "failed") {
    return (
      <svg
        className="w-5 h-5 text-red-400 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fillRule="evenodd"
          d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
          clipRule="evenodd"
        />
      </svg>
    );
  }

  return (
    <div className="w-5 h-5 rounded-full border-2 border-gray-600 flex-shrink-0" />
  );
}

interface ServerDetailsProps {
  instance: Instance;
  groupedRegions: RegionGroup[];
  isConnecting: boolean;
  isTerminating: boolean;
  vpnError: string | null;

  spawnJob?: SpawnJobState;
  onConnect: (data: Instance) => void;
  onTerminate: () => void;
}

export function ServerDetails({
  instance,
  groupedRegions,
  isConnecting,
  isTerminating,
  vpnError,
  spawnJob,
  onConnect,
  onTerminate,
}: ServerDetailsProps) {
  const getRegionFlag = (regionName?: string): string => {
    const region = groupedRegions
      .flatMap((g) => g.regions)
      .find((r) => r.name === regionName) as any;
    return region?.flag || "🌍";
  };

  const isSpawning = instance.state === "spawning";

  return (
    <div className="flex-1 flex flex-col bg-gray-900">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {}
          {!isSpawning && (
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <div className="flex items-center gap-3 mb-4">
                <span className="text-3xl">
                  {getRegionFlag(instance.region)}
                </span>
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
                  <p className="text-sm font-mono text-white break-all">
                    {instance.id}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-gray-400 mb-1">IPv4 Address</p>
                  <p className="text-sm font-mono text-white">
                    {isSpawning ? (
                      <span className="text-gray-500 italic">
                        Assigning IP address…
                      </span>
                    ) : (
                      instance.publicIpV4
                    )}
                  </p>
                </div>
                {instance.publicIpV6 && !isSpawning && (
                  <div>
                    <p className="text-xs text-gray-400 mb-1">IPv6 Address</p>
                    <p className="text-sm font-mono text-white">
                      {instance.publicIpV6}
                    </p>
                  </div>
                )}
              </div>
            </div>
          )}

          {}
          <div className="space-y-3">
            {isSpawning ? (
              <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
                <div className="px-4 py-3 border-b border-gray-700">
                  <p className="text-sm font-medium text-blue-400">
                    Deployment progress
                  </p>
                </div>
                <div className="p-3 space-y-1">
                  {spawnJob ? (
                    spawnJob.steps.map((step) => {
                      return (
                        <div
                          key={step.id}
                          className={`flex items-center gap-3 p-2.5 rounded-lg ${
                            step.status === "running"
                              ? "bg-blue-900/30 border border-blue-700/40"
                              : step.status === "completed"
                                ? "opacity-60"
                                : step.status === "failed"
                                  ? "bg-red-900/20 border border-red-700/40"
                                  : "opacity-40"
                          }`}
                        >
                          <StepIndicator status={step.status} />
                          <div className="flex-1 min-w-0">
                            <p
                              className={`text-sm ${
                                step.status === "running"
                                  ? "text-blue-300 font-medium"
                                  : step.status === "completed"
                                    ? "text-gray-400"
                                    : step.status === "failed"
                                      ? "text-red-300"
                                      : "text-gray-500"
                              }`}
                            >
                              {step.label}
                            </p>
                            {step.status === "failed" && step.error && (
                              <p className="text-xs text-red-400 mt-0.5 truncate">
                                {step.error}
                              </p>
                            )}
                          </div>
                        </div>
                      );
                    })
                  ) : (
                    <div className="flex items-center justify-center gap-2 py-4 text-blue-300">
                      <div className="w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
                      <span className="text-sm">Starting deployment…</span>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <>
                <button
                  onClick={() => onConnect(instance)}
                  disabled={isConnecting}
                  className="w-full px-6 py-4 bg-green-600 hover:bg-green-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isConnecting ? (
                    <div className="flex items-center justify-center gap-2">
                      <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                      Connecting…
                    </div>
                  ) : (
                    "Connect to VPN"
                  )}
                </button>

                <button
                  onClick={onTerminate}
                  disabled={isTerminating}
                  className="w-full px-6 py-4 bg-red-600 hover:bg-red-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isTerminating ? (
                    <div className="flex items-center justify-center gap-2">
                      <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                      Terminating…
                    </div>
                  ) : (
                    "Terminate Server"
                  )}
                </button>
              </>
            )}
          </div>

          {}
          {vpnError && (
            <div className="p-4 bg-red-900/20 border border-red-500/50 rounded-lg">
              <p className="text-red-300 text-sm">{vpnError}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
