import { useState } from "react";
import { Channel } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { invokeCommand } from "../lib/invokeCommand";
import {
  CloudProviderName,
  JobStep,
  JobStepState,
  JobStepStatus,
} from "../types";

enum ProvisionEventKind {
  Started = "STARTED",
  Progress = "PROGRESS",
  Complete = "COMPLETE",
  Failed = "FAILED",
}

type ProvisionAccountEvent =
  | { kind: ProvisionEventKind.Started; jobId: string; steps: JobStep[] }
  | {
      kind: ProvisionEventKind.Progress;
      stepId: string;
      status: JobStepStatus;
      error?: string;
    }
  | { kind: ProvisionEventKind.Complete; provider: CloudProviderName }
  | { kind: ProvisionEventKind.Failed; error: string };

export interface ProvisionJobState {
  jobId: string;
  provider: CloudProviderName;
  steps: JobStepState[];
}

interface UseAccountsOptions {
  onComplete?: (provider: CloudProviderName) => void;
  onFailed?: (error: string) => void;
}

export function useAccounts({ onComplete, onFailed }: UseAccountsOptions = {}) {
  const [activeProvisionJob, setActiveProvisionJob] =
    useState<ProvisionJobState | null>(null);
  const [isProvisionComplete, setIsProvisionComplete] = useState(false);
  const [provisionError, setProvisionError] = useState<string | null>(null);

  async function provisionAccount(provider: CloudProviderName) {
    const onEvent = new Channel<ProvisionAccountEvent>();
    onEvent.onmessage = (event) => applyProvisionEvent(event, provider);

    resetProvisionState();

    try {
      await invokeCommand("provision_account", { provider, onEvent });
    } catch (error) {
      toast.error(extractErrorMessage(error, "Failed to start provisioning"));
    }
  }

  function applyProvisionEvent(
    event: ProvisionAccountEvent,
    provider: CloudProviderName,
  ) {
    switch (event.kind) {
      case ProvisionEventKind.Started:
        setActiveProvisionJob({
          jobId: event.jobId,
          provider,
          steps: event.steps.map((step) => ({
            ...step,
            status: JobStepStatus.Pending,
          })),
        });
        return;

      case ProvisionEventKind.Progress:
        setActiveProvisionJob((previous) =>
          previous
            ? {
                ...previous,
                steps: previous.steps.map((step) =>
                  step.id === event.stepId
                    ? { ...step, status: event.status, error: event.error }
                    : step,
                ),
              }
            : previous,
        );
        return;

      case ProvisionEventKind.Complete:
        setIsProvisionComplete(true);
        onComplete?.(event.provider);
        return;

      case ProvisionEventKind.Failed:
        setProvisionError(event.error);
        onFailed?.(event.error);
        return;
    }
  }

  function resetProvisionState() {
    setActiveProvisionJob(null);
    setIsProvisionComplete(false);
    setProvisionError(null);
  }

  return {
    activeProvisionJob,
    isProvisionComplete,
    provisionError,
    provisionAccount,
    resetProvisionState,
  };
}
