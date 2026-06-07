import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import { invokeCommand } from "../lib/invokeCommand";
import {
  CloudProviderName,
  ProvisionAccountCompleteEvent,
  ProvisionAccountJob,
  ProvisionAccountProgressEvent,
  ProvisionJobState,
  SpawnStepState,
  SpawnStepStatus,
} from "../types";

enum ProvisionEvent {
  Progress = "provision-account-progress",
  Complete = "provision-account-complete",
  Failed = "provision-account-failed",
}

interface UseAccountsOptions {
  onComplete?: (provider: CloudProviderName) => void;
  onFailed?: (error: string) => void;
}

export function useAccounts({ onComplete, onFailed }: UseAccountsOptions = {}) {
  const [activeProvisionJob, setActiveProvisionJob] = useState<ProvisionJobState | null>(null);
  const [isProvisionDrawerOpen, setIsProvisionDrawerOpen] = useState(false);
  const [isProvisionComplete, setIsProvisionComplete] = useState(false);
  const [provisionError, setProvisionError] = useState<string | null>(null);
  const activeJobIdRef = useRef<string | null>(null);
  const earlyProgressEventsRef = useRef<ProvisionAccountProgressEvent[]>([]);

  useEffect(() => {
    const progressUnlisten = listen<ProvisionAccountProgressEvent>(
      ProvisionEvent.Progress,
      ({ payload }) => {
        const { jobId, stepId, status, error: stepError } = payload;
        setActiveProvisionJob((previous) => {
          if (!previous || previous.jobId !== jobId) {
            earlyProgressEventsRef.current.push(payload);
            return previous;
          }
          return {
            ...previous,
            steps: previous.steps.map((step) =>
              step.id === stepId ? { ...step, status, error: stepError } : step,
            ),
          };
        });
      },
    );

    const completeUnlisten = listen<ProvisionAccountCompleteEvent>(
      ProvisionEvent.Complete,
      ({ payload }) => {
        if (activeJobIdRef.current === payload.jobId) {
          setIsProvisionComplete(true);
          onComplete?.(payload.provider);
        }
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      ProvisionEvent.Failed,
      ({ payload }) => {
        if (activeJobIdRef.current === payload.jobId) {
          setProvisionError(payload.error);
          onFailed?.(payload.error);
        }
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const setupNewAccount = async (provider: CloudProviderName) => {
    try {
      earlyProgressEventsRef.current = [];
      const job = await invokeCommand<ProvisionAccountJob>("provision_account", { provider });
      const bufferedEvents = earlyProgressEventsRef.current.filter((event) => event.jobId === job.jobId);
      earlyProgressEventsRef.current = [];
      const initialSteps: SpawnStepState[] = job.steps.map((step) => {
        const latestBufferedEvent = [...bufferedEvents].reverse().find((event) => event.stepId === step.id);
        return {
          ...step,
          status: latestBufferedEvent?.status ?? SpawnStepStatus.Pending,
          error: latestBufferedEvent?.error,
        };
      });
      activeJobIdRef.current = job.jobId;
      setActiveProvisionJob({ jobId: job.jobId, provider, steps: initialSteps });
      setIsProvisionComplete(false);
      setProvisionError(null);
      setIsProvisionDrawerOpen(true);
    } catch (invocationError) {
      const message =
        invocationError instanceof Error ? invocationError.message : "Failed to start provisioning";
      toast.error(message);
    }
  };

  const closeProvisionDrawer = () => setIsProvisionDrawerOpen(false);

  return {
    activeProvisionJob,
    isProvisionDrawerOpen,
    isProvisionComplete,
    provisionError,
    setupNewAccount,
    closeProvisionDrawer,
  };
}
