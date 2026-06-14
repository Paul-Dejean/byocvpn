import { useEffect, useState } from "react";
import {
  Instance,
  InstanceState,
  SpawnJobState,
  JobStepStatus,
} from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../primitives/Spinner";
import { Card } from "../primitives/Card";
import { Alert } from "../primitives/Alert";
import { Button } from "../primitives/Button";
import { formatUptime } from "../../lib/time";

function StepIndicator({ status }: { status: JobStepStatus }) {
  if (status === JobStepStatus.Running) {
    return <Spinner size="w-5 h-5" color="border-blue-400" />;
  }
  if (status === JobStepStatus.Completed) {
    return (
      <svg
        className="w-5 h-5 text-success-400 flex-shrink-0"
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
  if (status === JobStepStatus.Failed) {
    return (
      <svg
        className="w-5 h-5 text-danger-400 flex-shrink-0"
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
  const isInProgress =
    isSpawning || instance.state === InstanceState.Installing;
  const isFailedSpawn =
    instance.state === InstanceState.Error && spawnJob !== undefined;

  const startTime = instance.launchedAt
    ? new Date(instance.launchedAt).getTime()
    : null;
  const [elapsedSeconds, setElapsedSeconds] = useState(
    startTime ? Math.floor((Date.now() - startTime) / 1000) : 0
  );

  useEffect(() => {
    if (!startTime) return;
    const interval = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);
    return () => clearInterval(interval);
  }, [startTime]);

  const uptimeHours = elapsedSeconds / 3600;
  // const estimatedCost = pricing
  //   ? (pricing.hourlyRate + pricing.ipHourlyRate) * uptimeHours
  //   : null;

  return (
    <div className="flex-1 min-w-0 flex flex-col bg-gray-900">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {!isSpawning && (
            <Card>
              <div className="flex items-center gap-3 mb-4">
                <FlagIcon
                  countryCode={regionInfo.countryCode}
                  className="text-3xl"
                />
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-blue-400">
                    {regionInfo.city}
                  </h3>
                  <p className="text-sm text-gray-400 font-mono">
                    {instance.region}
                  </p>
                </div>
                <ProviderIcon
                  provider={instance.provider}
                  className="w-8 h-8"
                />
              </div>
              <div className="space-y-3">
                <div>
                  <p className="text-xs text-gray-400 mb-1">Instance ID</p>
                  <p className="text-sm font-mono text-primary break-all">
                    {instance.id}
                  </p>
                </div>
                {instance.instanceType && (
                  <div>
                    <p className="text-xs text-gray-400 mb-1">Instance Type</p>
                    <p className="text-sm font-mono text-primary">{instance.instanceType}</p>
                  </div>
                )}
                <div>
                  <p className="text-xs text-gray-400 mb-1">IPv4 Address</p>
                  <p className="text-sm font-mono text-primary">
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
                    <p className="text-sm font-mono text-primary">
                      {instance.publicIpV6}
                    </p>
                  </div>
                )}
                {instance.launchedAt && (
                  <div>
                    <p className="text-xs text-gray-400 mb-1">Uptime</p>
                    <p className="text-sm font-mono text-primary">{formatUptime(uptimeHours)}</p>
                  </div>
                )}
                {/* {estimatedCost !== null && (
                  <div>
                    <p className="text-xs text-gray-400 mb-1">Est. Cost</p>
                    <p className="text-sm font-mono text-primary">${estimatedCost.toFixed(6)}</p>
                  </div>
                )} */}
              </div>
            </Card>
          )}

          <div className="space-y-3">
            {isInProgress || isFailedSpawn ? (
              <Card padded={false} className="overflow-hidden">
                <div className="px-4 py-3 border-b border-gray-700/50">
                  <p
                    className={`text-sm font-medium ${isFailedSpawn ? "text-danger-400" : "text-blue-400"}`}
                  >
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
                            step.status === JobStepStatus.Running
                              ? "bg-blue-900/30 border border-blue-700/40"
                              : step.status === JobStepStatus.Completed
                                ? "opacity-60"
                                : step.status === JobStepStatus.Failed
                                  ? "bg-danger-900/20 border border-danger-700/40"
                                  : "opacity-40"
                          }`}
                        >
                          <StepIndicator status={step.status} />
                          <div className="flex-1 min-w-0">
                            <p
                              className={`text-sm ${
                                step.status === JobStepStatus.Running
                                  ? "text-blue-300 font-medium"
                                  : step.status === JobStepStatus.Completed
                                    ? "text-gray-400"
                                    : step.status === JobStepStatus.Failed
                                      ? "text-danger-300"
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
              </Card>
            ) : (
              <>
                <Button
                  variant="primary"
                  size="lg"
                  disabledStyle="dim"
                  loading={isConnecting}
                  onClick={() => onConnect(instance)}
                  className="w-full"
                >
                  {isConnecting ? "Connecting…" : "Connect to VPN"}
                </Button>

                <Button
                  variant="danger"
                  size="lg"
                  disabledStyle="dim"
                  loading={isTerminating}
                  onClick={onTerminate}
                  className="w-full"
                >
                  {isTerminating ? "Terminating…" : "Terminate Server"}
                </Button>
              </>
            )}
            {isFailedSpawn && instance.errorReason && (
              <Alert
                variant="error"
                title="Deployment failed"
                icon={
                  <svg
                    className="w-5 h-5 text-danger-400"
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
                }
              >
                {instance.errorReason}
              </Alert>
            )}
            {isFailedSpawn && (
              <Button
                variant="secondary"
                size="lg"
                onClick={onDismiss}
                className="w-full"
              >
                Dismiss
              </Button>
            )}
          </div>

          {vpnError && <Alert variant="error">{vpnError}</Alert>}
        </div>
      </div>
    </div>
  );
}
