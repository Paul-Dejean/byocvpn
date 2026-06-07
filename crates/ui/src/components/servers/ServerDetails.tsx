import {
  Instance,
  InstanceState,
  SpawnJobState,
  SpawnStepStatus,
} from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../common/Spinner";

function StepIndicator({ status }: { status: SpawnStepStatus }) {
  if (status === SpawnStepStatus.Running) {
    return <Spinner size="w-5 h-5" color="border-blue-400" />;
  }
  if (status === SpawnStepStatus.Completed) {
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
  if (status === SpawnStepStatus.Failed) {
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
  isConnecting: boolean;
  isTerminating: boolean;
  vpnError: string | null;
  spawnJob?: SpawnJobState;
  onConnect: (data: Instance) => void;
  onTerminate: () => void;
  onDismiss: () => void;
}

export function ServerDetails({
  instance,
  isConnecting,
  isTerminating,
  vpnError,
  spawnJob,
  onConnect,
  onTerminate,
  onDismiss,
}: ServerDetailsProps) {
  const regionInfo = getRegionInfo(instance.provider, instance.region ?? "");
  const isSpawning = instance.state === InstanceState.Spawning;
  const isInProgress = isSpawning || instance.state === InstanceState.Installing;
  const isFailedSpawn = instance.state === InstanceState.Error && spawnJob !== undefined;

  return (
    <div className="flex-1 min-w-0 flex flex-col bg-gray-900">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {!isSpawning && (
            <div className="bg-gray-800 rounded-lg p-6">
              <div className="flex items-center gap-3 mb-4">
                <FlagIcon countryCode={regionInfo.countryCode} className="text-3xl" />
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-blue-400">
                    {regionInfo.city}
                  </h3>
                  <p className="text-sm text-gray-400 font-mono">{instance.region}</p>
                </div>
                <ProviderIcon provider={instance.provider} className="w-8 h-8" />
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

          <div className="space-y-3">
            {(isInProgress || isFailedSpawn) ? (
              <div className="bg-gray-800 rounded-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-gray-700/50">
                  <p className={`text-sm font-medium ${isFailedSpawn ? "text-red-400" : "text-blue-400"}`}>
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
                            step.status === SpawnStepStatus.Running
                              ? "bg-blue-900/30 border border-blue-700/40"
                              : step.status === SpawnStepStatus.Completed
                                ? "opacity-60"
                                : step.status === SpawnStepStatus.Failed
                                  ? "bg-red-900/20 border border-red-700/40"
                                  : "opacity-40"
                          }`}
                        >
                          <StepIndicator status={step.status} />
                          <div className="flex-1 min-w-0">
                            <p
                              className={`text-sm ${
                                step.status === SpawnStepStatus.Running
                                  ? "text-blue-300 font-medium"
                                  : step.status === SpawnStepStatus.Completed
                                    ? "text-gray-400"
                                    : step.status === SpawnStepStatus.Failed
                                      ? "text-red-300"
                                      : "text-gray-500"
                              }`}
                            >
                              {step.label}
                            </p>
                          </div>
                        </div>
                      );
                    })
                  ) : (
                    <div className="flex items-center justify-center gap-2 py-4 text-blue-300">
                      <Spinner size="w-5 h-5" color="border-blue-400" />
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
                  className="btn-primary w-full px-6 py-4 text-lg disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isConnecting ? (
                    <div className="flex items-center justify-center gap-2">
                      <Spinner size="w-5 h-5" color="border-white" />
                      Connecting…
                    </div>
                  ) : (
                    "Connect to VPN"
                  )}
                </button>

                <button
                  onClick={onTerminate}
                  disabled={isTerminating}
                  className="btn-danger w-full px-6 py-4 text-lg disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isTerminating ? (
                    <div className="flex items-center justify-center gap-2">
                      <Spinner size="w-5 h-5" color="border-white" />
                      Terminating…
                    </div>
                  ) : (
                    "Terminate Server"
                  )}
                </button>
              </>
            )}
            {isFailedSpawn && instance.errorReason && (
              <div className="flex items-start gap-3 p-4 bg-red-900/50 border border-red-700 rounded-lg">
                <svg
                  className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
                <div>
                  <p className="text-red-300 text-sm font-medium">Deployment failed</p>
                  <p className="text-xs text-red-400/70 mt-0.5 break-all">{instance.errorReason}</p>
                </div>
              </div>
            )}
            {isFailedSpawn && (
              <button
                onClick={onDismiss}
                className="btn-secondary w-full px-6 py-4 text-lg"
              >
                Dismiss
              </button>
            )}
          </div>

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
