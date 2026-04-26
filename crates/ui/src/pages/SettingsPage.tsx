import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { load as loadStore } from "@tauri-apps/plugin-store";
import toast from "react-hot-toast";
import { useCredentials } from "../hooks/useCredentials";
import { OracleProfileCard } from "../components/settings/OracleProfileCard";
import { GcpProfileCard } from "../components/settings/GcpProfileCard";
import { AzureProfileCard } from "../components/settings/AzureProfileCard";
import { ProvisionAccountDrawer } from "../components/settings/ProvisionAccountDrawer";
import { NotificationSettingsCard } from "../components/settings/NotificationSettingsCard";
import {
  ProvisionAccountJob,
  ProvisionAccountProgressEvent,
  ProvisionAccountCompleteEvent,
  ProvisionJobState,
  SpawnStepState,
} from "../types";

interface SettingsPageProps {
  onNavigateToAddAccount?: () => void;
}

export function SettingsPage({ onNavigateToAddAccount }: SettingsPageProps) {
  const [isAwsEditing, setIsAwsEditing] = useState(false);
  const [awsHasCredentials, setAwsHasCredentials] = useState<boolean | null>(null);
  const [oracleHasCredentials, setOracleHasCredentials] = useState<boolean | null>(null);
  const [gcpHasCredentials, setGcpHasCredentials] = useState<boolean | null>(null);
  const [azureHasCredentials, setAzureHasCredentials] = useState<boolean | null>(null);
  const [isAwsConfirmingDelete, setIsAwsConfirmingDelete] = useState(false);

  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");

  const [activeProvisionJob, setActiveProvisionJob] =
    useState<ProvisionJobState | null>(null);
  const [isProvisionDrawerOpen, setIsProvisionDrawerOpen] = useState(false);
  const [isProvisionComplete, setIsProvisionComplete] = useState(false);
  const [provisionError, setProvisionError] = useState<string | null>(null);

  const [provisionedProviders, setProvisionedProviders] = useState<Set<string>>(new Set());

  const activeJobIdRef = useRef<string | null>(null);

  const { isSaving, error, successMessage, saveCredentials, deleteCredentials, loadCredentials } =
    useCredentials();

  useEffect(() => {
    loadCredentials("aws").then((existing) => setAwsHasCredentials(existing !== null));
    loadCredentials("oracle").then((existing) => setOracleHasCredentials(existing !== null));
    loadCredentials("gcp").then((existing) => setGcpHasCredentials(existing !== null));
    loadCredentials("azure").then((existing) => setAzureHasCredentials(existing !== null));
  }, []);

  useEffect(() => {
    const fetchProvisionedProviders = async () => {
      const store = await loadStore("providers.json");
      const provisioned = new Set<string>();
      for (const provider of ["aws", "oracle", "gcp", "azure"]) {
        const value = await store.get<boolean>(`provisioned/${provider}`);
        if (value === true) {
          provisioned.add(provider);
        }
      }
      setProvisionedProviders(provisioned);
    };
    fetchProvisionedProviders();
  }, []);

  useEffect(() => {
    const progressUnlisten = listen<ProvisionAccountProgressEvent>(
      "provision-account-progress",
      ({ payload }) => {
        const { jobId, stepId, status, error: stepError } = payload;
        setActiveProvisionJob((previous) => {
          if (!previous || previous.jobId !== jobId) return previous;
          return {
            ...previous,
            steps: previous.steps.map((step) =>
              step.id === stepId
                ? { ...step, status, error: stepError }
                : step,
            ),
          };
        });
      },
    );

    const completeUnlisten = listen<ProvisionAccountCompleteEvent>(
      "provision-account-complete",
      ({ payload }) => {
        if (activeJobIdRef.current === payload.jobId) {
          setIsProvisionComplete(true);
          setProvisionedProviders((previous) => new Set([...previous, payload.provider]));
          toast.success("Account provisioned successfully!");
        }
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      "provision-account-failed",
      ({ payload }) => {
        if (activeJobIdRef.current === payload.jobId) {
          setProvisionError(payload.error);
          toast.error("Provisioning failed");
        }
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const startProvisionAccount = async (provider: string) => {
    try {
      const job = await invoke<ProvisionAccountJob>("provision_account", {
        provider,
      });

      const initialSteps: SpawnStepState[] = job.steps.map((step) => ({
        ...step,
        status: "pending" as const,
      }));

      activeJobIdRef.current = job.jobId;
      setActiveProvisionJob({
        jobId: job.jobId,
        provider,
        steps: initialSteps,
      });
      setIsProvisionComplete(false);
      setProvisionError(null);
      setIsProvisionDrawerOpen(true);
    } catch (invocationError) {
      const message =
        invocationError instanceof Error
          ? invocationError.message
          : "Failed to start provisioning";
      toast.error(message);
    }
  };

  const handleAwsEditOpen = async () => {
    const existing = await loadCredentials("aws");
    if (existing) {
      setAccessKey((existing as { accessKeyId: string }).accessKeyId);
    }
    setIsAwsEditing(true);
  };

  const handleAwsCancelEdit = () => {
    setIsAwsEditing(false);
    setAccessKey("");
    setSecretKey("");
  };

  const handleAwsSaveProfile = async () => {
    if (!accessKey.trim()) return;

    const success = await saveCredentials("aws", {
      accessKeyId: accessKey.trim(),
      secretAccessKey: secretKey.trim(),
    });

    if (success) {
      setAccessKey("");
      setSecretKey("");
      setIsAwsEditing(false);
      setAwsHasCredentials(true);
      startProvisionAccount("aws");
    }
  };

  const handleAwsDeleteCredentials = async () => {
    const success = await deleteCredentials("aws");
    if (success) {
      setAwsHasCredentials(false);
      setIsAwsConfirmingDelete(false);
      setIsAwsEditing(false);
      setProvisionedProviders((previous) => {
        const next = new Set(previous);
        next.delete("aws");
        return next;
      });
    }
  };

  const handleCloseProvisionDrawer = () => {
    setIsProvisionDrawerOpen(false);
  };

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto">
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-6 text-blue-400">
              Manage Profiles
            </h2>

            {awsHasCredentials === true && (
              <div className={`rounded-lg p-6 ${!provisionedProviders.has("aws") ? "bg-gray-700 border-l-4 border-amber-500" : "bg-gray-700"}`}>
                {!isAwsEditing ? (
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-white/5 rounded-lg flex items-center justify-center p-2">
                        <img src="/cloud-providers/aws-icon.svg" alt="AWS" className="w-full h-full object-contain" />
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <h3 className="font-semibold text-lg text-white">AWS Profile</h3>
                          {provisionedProviders.has("aws") ? (
                            <span className="text-xs px-2 py-0.5 bg-green-900/50 text-green-400 rounded-full border border-green-700/50">
                              Provisioned
                            </span>
                          ) : (
                            <span className="text-xs px-2 py-0.5 bg-yellow-900/50 text-yellow-300 rounded-full border border-yellow-700/50">
                              Not provisioned
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-gray-400">
                          Amazon Web Services credentials for EC2 deployment
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {isAwsConfirmingDelete ? (
                        <>
                          <span className="text-sm text-gray-300">Delete?</span>
                          <button onClick={() => setIsAwsConfirmingDelete(false)} className="px-3 py-1.5 btn-secondary text-sm">Cancel</button>
                          <button onClick={handleAwsDeleteCredentials} className="px-3 py-1.5 btn-danger text-sm">Confirm</button>
                        </>
                      ) : (
                        <>
                          {provisionedProviders.has("aws") ? (
                            <button onClick={() => startProvisionAccount("aws")} className="p-2 text-gray-400 hover:text-blue-400 hover:bg-gray-600 rounded-lg transition-colors" title="Re-provision">
                              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                              </svg>
                            </button>
                          ) : (
                            <button onClick={() => startProvisionAccount("aws")} className="p-2 text-amber-400 hover:text-amber-300 hover:bg-gray-600 rounded-lg transition-colors" title="Provision">
                              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                              </svg>
                            </button>
                          )}
                          <button onClick={() => setIsAwsConfirmingDelete(true)} className="p-2 text-gray-400 hover:text-red-400 hover:bg-gray-600 rounded-lg transition-colors" title="Delete credentials">
                            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </button>
                          <button onClick={handleAwsEditOpen} className="px-4 py-2 btn-primary font-medium flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                            </svg>
                            Edit
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                ) : (
                  <div className="space-y-6">
                    <div className="flex items-center gap-4 mb-6">
                      <div className="w-12 h-12 bg-white/5 rounded-lg flex items-center justify-center p-2">
                        <img src="/cloud-providers/aws-icon.svg" alt="AWS" className="w-full h-full object-contain" />
                      </div>
                      <div>
                        <h3 className="font-semibold text-lg text-white">Edit AWS Profile</h3>
                        <p className="text-sm text-gray-400">Update your AWS access credentials</p>
                      </div>
                    </div>
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-300 mb-1">Access Key ID</label>
                        <p className="text-xs text-gray-500 mb-2">e.g. AKIAIOSFODNN7EXAMPLE</p>
                        <input
                          type="text"
                          value={accessKey}
                          onChange={(e) => setAccessKey(e.target.value)}
                          className="input"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-300 mb-1">Secret Access Key</label>
                        <p className="text-xs text-gray-500 mb-2">Leave blank to keep your existing key</p>
                        <input
                          type="password"
                          value={secretKey}
                          onChange={(e) => setSecretKey(e.target.value)}
                          className="input"
                        />
                      </div>
                      {error && (
                        <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
                          <p className="text-red-300 text-sm">{error}</p>
                        </div>
                      )}
                      {successMessage && (
                        <div className="p-3 bg-green-900 border border-green-700 rounded-lg">
                          <p className="text-green-300 text-sm">{successMessage}</p>
                        </div>
                      )}
                      <div className="flex gap-3 pt-4">
                        <button onClick={handleAwsCancelEdit} className="flex-1 px-4 py-2 btn-secondary">
                          Cancel
                        </button>
                        <button
                          onClick={handleAwsSaveProfile}
                          disabled={isSaving || !accessKey.trim()}
                          className="btn-primary flex-1 px-4 py-2 disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-600"
                        >
                          {isSaving ? (
                            <div className="flex items-center justify-center gap-2">
                              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                              Saving...
                            </div>
                          ) : (
                            "Save Profile"
                          )}
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {oracleHasCredentials === true && (
              <OracleProfileCard
                onCredentialsSaved={startProvisionAccount}
                onProvisionRequested={startProvisionAccount}
                isProvisioned={provisionedProviders.has("oracle")}
                onCredentialsDeleted={() => {
                  setOracleHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete("oracle");
                    return next;
                  });
                }}
              />
            )}

            {gcpHasCredentials === true && (
              <GcpProfileCard
                onCredentialsSaved={startProvisionAccount}
                onProvisionRequested={startProvisionAccount}
                isProvisioned={provisionedProviders.has("gcp")}
                onCredentialsDeleted={() => {
                  setGcpHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete("gcp");
                    return next;
                  });
                }}
              />
            )}

            {azureHasCredentials === true && (
              <AzureProfileCard
                onCredentialsSaved={startProvisionAccount}
                onProvisionRequested={startProvisionAccount}
                isProvisioned={provisionedProviders.has("azure")}
                onCredentialsDeleted={() => {
                  setAzureHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete("azure");
                    return next;
                  });
                }}
              />
            )}

            <div className="mt-4 pt-4 border-t border-gray-600">
              <NotificationSettingsCard />
            </div>

            {onNavigateToAddAccount && (
              <div className="mt-4 pt-4 border-t border-gray-600">
                <button
                  onClick={onNavigateToAddAccount}
                  className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-gray-700 hover:bg-gray-600 border border-dashed border-gray-500 hover:border-gray-400 text-gray-300 hover:text-white rounded-lg transition-colors font-medium"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                  </svg>
                  Add Account
                </button>
              </div>
            )}
          </div>

        </div>
      </div>

      <ProvisionAccountDrawer
        isOpen={isProvisionDrawerOpen}
        onClose={handleCloseProvisionDrawer}
        provider={activeProvisionJob?.provider ?? ""}
        steps={activeProvisionJob?.steps ?? []}
        isComplete={isProvisionComplete}
        error={provisionError}
      />
    </div>
  );
}
