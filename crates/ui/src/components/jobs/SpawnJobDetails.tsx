import { SpawnJob, SpawnJobStatus } from "../../types";
import { Card } from "../primitives/Card";
import { Alert } from "../primitives/Alert";
import { Button } from "../primitives/Button";
import { JobStepList } from "./JobStepList";

interface SpawnJobDetailsProps {
  spawnJob: SpawnJob;
  onDismiss: (jobId: string) => void;
}

export function SpawnJobDetails({ spawnJob, onDismiss }: SpawnJobDetailsProps) {
  const hasFailed = spawnJob.status === SpawnJobStatus.Failed;

  return (
    <div className="flex-1 min-w-0 flex flex-col bg-gray-900">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          <div className="space-y-3">
            <Card padded={false} className="overflow-hidden">
              <div className="px-4 py-3 border-b border-gray-700/50">
                <p
                  className={`text-sm font-medium ${hasFailed ? "text-danger-400" : "text-blue-400"}`}
                >
                  Deployment progress
                </p>
              </div>
              <JobStepList steps={spawnJob.steps} />
            </Card>

            {hasFailed && spawnJob.error && (
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
                {spawnJob.error}
              </Alert>
            )}

            {hasFailed && (
              <Button
                variant="secondary"
                size="lg"
                onClick={() => onDismiss(spawnJob.jobId)}
                className="w-full"
              >
                Dismiss
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
