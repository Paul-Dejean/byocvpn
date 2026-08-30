import { useEffect, useState } from "react";
import { Instance, InstanceState, SpawnJob } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { FlagIcon } from "../FlagIcon";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../primitives/Spinner";
import { Card } from "../primitives/Card";
import { Alert } from "../primitives/Alert";
import { Button } from "../primitives/Button";
import { formatUptime } from "../../lib/time";
import { JobStepList } from "../jobs/JobStepList";

interface ServerDetailsProps {
  instance: Instance;
  isConnecting: boolean;
  isTerminating: boolean;
  vpnError: string | null;
  spawnJob?: SpawnJob;
  onConnect: (data: Instance) => void;
  onTerminate: () => void;
}

export function ServerDetails({
  instance,
  isConnecting,
  isTerminating,
  vpnError,
  spawnJob,
  onConnect,
  onTerminate,
}: ServerDetailsProps) {
  const regionInfo = getRegionInfo(instance.provider, instance.region ?? "");
  const isInstalling = instance.state === InstanceState.Installing;

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
                    {instance.publicIpV4}
                  </p>
                </div>
                {instance.publicIpV6 && (
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

          <div className="space-y-3">
            {isInstalling ? (
              <Card padded={false} className="overflow-hidden">
                <div className="px-4 py-3 border-b border-gray-700/50">
                  <p className="text-sm font-medium text-blue-400">
                    Deployment progress
                  </p>
                </div>
                {spawnJob ? (
                  <JobStepList steps={spawnJob.steps} />
                ) : (
                  <div className="flex items-center justify-center gap-2 py-4 text-blue-300">
                    <Spinner size="w-5 h-5" color="border-blue-400" />
                    <span className="text-sm">Finishing setup…</span>
                  </div>
                )}
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
          </div>

          {instance.state === InstanceState.Error && instance.errorReason && (
            <Alert variant="error">{instance.errorReason}</Alert>
          )}
          {vpnError && <Alert variant="error">{vpnError}</Alert>}
        </div>
      </div>
    </div>
  );
}
