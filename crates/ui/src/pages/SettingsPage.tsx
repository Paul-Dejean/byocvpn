import { useEffect, useState } from "react";
import { load as loadStore } from "@tauri-apps/plugin-store";
import toast from "react-hot-toast";
import { useCredentials } from "../hooks/useCredentials";
import { useAccounts } from "../hooks/useAccounts";
import { CloudProviderName } from "../types";
import { AwsProfileCard } from "../components/settings/AwsProfileCard";
import { OracleProfileCard } from "../components/settings/OracleProfileCard";
import { GcpProfileCard } from "../components/settings/GcpProfileCard";
import { AzureProfileCard } from "../components/settings/AzureProfileCard";
import { StepProgressDrawer } from "../components/settings/StepProgressDrawer";
import { NotificationSettingsCard } from "../components/settings/NotificationSettingsCard";
import { KillSwitchSettingsCard } from "../components/settings/KillSwitchSettingsCard";

interface SettingsPageProps {
  onNavigateToAddAccount?: () => void;
}

export function SettingsPage({ onNavigateToAddAccount }: SettingsPageProps) {
  const [awsHasCredentials, setAwsHasCredentials] = useState<boolean | null>(null);
  const [oracleHasCredentials, setOracleHasCredentials] = useState<boolean | null>(null);
  const [gcpHasCredentials, setGcpHasCredentials] = useState<boolean | null>(null);
  const [azureHasCredentials, setAzureHasCredentials] = useState<boolean | null>(null);

  const [provisionedProviders, setProvisionedProviders] = useState<Set<CloudProviderName>>(new Set());

  const { loadCredentials } = useCredentials();

  const {
    activeProvisionJob,
    isProvisionDrawerOpen,
    isProvisionComplete,
    provisionError,
    provisionAccount,
    closeProvisionDrawer,
  } = useAccounts({
    onComplete: (provider) => {
      setProvisionedProviders((previous) => new Set([...previous, provider]));
      toast.success("Account provisioned successfully!");
    },
    onFailed: () => toast.error("Provisioning failed"),
  });

  useEffect(() => {
    loadCredentials(CloudProviderName.Aws).then((existing) =>
      setAwsHasCredentials(existing !== null),
    );
    loadCredentials(CloudProviderName.Oracle).then((existing) =>
      setOracleHasCredentials(existing !== null),
    );
    loadCredentials(CloudProviderName.Gcp).then((existing) =>
      setGcpHasCredentials(existing !== null),
    );
    loadCredentials(CloudProviderName.Azure).then((existing) =>
      setAzureHasCredentials(existing !== null),
    );
  }, []);

  useEffect(() => {
    const fetchProvisionedProviders = async () => {
      const store = await loadStore("providers.json");
      const provisioned = new Set<CloudProviderName>();
      for (const provider of Object.values(CloudProviderName)) {
        const value = await store.get<boolean>(`provisioned/${provider}`);
        if (value === true) {
          provisioned.add(provider);
        }
      }
      setProvisionedProviders(provisioned);
    };
    fetchProvisionedProviders();
  }, []);

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto">
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-6 text-blue-400">
              Manage Profiles
            </h2>

            {awsHasCredentials === true && (
              <AwsProfileCard
                onCredentialsSaved={provisionAccount}
                onProvisionRequested={provisionAccount}
                isProvisioned={provisionedProviders.has(CloudProviderName.Aws)}
                onCredentialsDeleted={() => {
                  setAwsHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete(CloudProviderName.Aws);
                    return next;
                  });
                }}
              />
            )}

            {oracleHasCredentials === true && (
              <OracleProfileCard
                onCredentialsSaved={provisionAccount}
                onProvisionRequested={provisionAccount}
                isProvisioned={provisionedProviders.has(
                  CloudProviderName.Oracle,
                )}
                onCredentialsDeleted={() => {
                  setOracleHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete(CloudProviderName.Oracle);
                    return next;
                  });
                }}
              />
            )}

            {gcpHasCredentials === true && (
              <GcpProfileCard
                onCredentialsSaved={provisionAccount}
                onProvisionRequested={provisionAccount}
                isProvisioned={provisionedProviders.has(CloudProviderName.Gcp)}
                onCredentialsDeleted={() => {
                  setGcpHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete(CloudProviderName.Gcp);
                    return next;
                  });
                }}
              />
            )}

            {azureHasCredentials === true && (
              <AzureProfileCard
                onCredentialsSaved={provisionAccount}
                onProvisionRequested={provisionAccount}
                isProvisioned={provisionedProviders.has(
                  CloudProviderName.Azure,
                )}
                onCredentialsDeleted={() => {
                  setAzureHasCredentials(false);
                  setProvisionedProviders((previous) => {
                    const next = new Set(previous);
                    next.delete(CloudProviderName.Azure);
                    return next;
                  });
                }}
              />
            )}

            <div className="mt-4 pt-4 border-t border-gray-600">
              <NotificationSettingsCard />
            </div>

            <div className="mt-4 pt-4 border-t border-gray-600">
              <KillSwitchSettingsCard />
            </div>

            {onNavigateToAddAccount && (
              <div className="mt-4 pt-4 border-t border-gray-600">
                <button
                  onClick={onNavigateToAddAccount}
                  className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-gray-700 hover:bg-gray-600 border border-dashed border-gray-500 hover:border-gray-400 text-gray-300 hover:text-white rounded-lg transition-colors font-medium"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 4v16m8-8H4"
                    />
                  </svg>
                  Add Account
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <StepProgressDrawer
        isOpen={isProvisionDrawerOpen}
        onClose={closeProvisionDrawer}
        provider={activeProvisionJob?.provider ?? CloudProviderName.Aws}
        steps={activeProvisionJob?.steps ?? []}
        isComplete={isProvisionComplete}
        error={provisionError}
      />
    </div>
  );
}
