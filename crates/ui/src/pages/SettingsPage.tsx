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
import { JobProgressDrawer } from "../components/common/JobProgressDrawer";
import { NotificationSettingsCard } from "../components/settings/NotificationSettingsCard";
import { SessionKillswitchCard } from "../components/settings/SessionKillswitchCard";

interface SettingsPageProps {
  onNavigateToAddAccount?: () => void;
}

export function SettingsPage({ onNavigateToAddAccount }: SettingsPageProps) {
  const [awsHasCredentials, setAwsHasCredentials] = useState<boolean | null>(
    null,
  );
  const [oracleHasCredentials, setOracleHasCredentials] = useState<
    boolean | null
  >(null);
  const [gcpHasCredentials, setGcpHasCredentials] = useState<boolean | null>(
    null,
  );
  const [azureHasCredentials, setAzureHasCredentials] = useState<
    boolean | null
  >(null);

  const [provisionedProviders, setProvisionedProviders] = useState<
    Set<CloudProviderName>
  >(new Set());

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
      <div className="flex-1 overflow-y-auto">
        <div className="px-6 pb-8">
          <div>
            <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-500 pt-6 pb-2 border-b border-gray-700/50">
              Accounts
            </h2>
            <div className="divide-y divide-gray-700/50">
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
                  isProvisioned={provisionedProviders.has(CloudProviderName.Oracle)}
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
                  isProvisioned={provisionedProviders.has(CloudProviderName.Azure)}
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

              {onNavigateToAddAccount && !(awsHasCredentials && oracleHasCredentials && gcpHasCredentials && azureHasCredentials) && (
                <div className="py-4">
                  <button
                    onClick={onNavigateToAddAccount}
                    className="flex items-center gap-2 px-6 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl transition-colors font-semibold"
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

          <div className="mt-8">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-500 pb-2 border-b border-gray-700/50">
              VPN Settings
            </h2>
            <div className="divide-y divide-gray-700/50">
              <SessionKillswitchCard />
              <NotificationSettingsCard />
            </div>
          </div>
        </div>
      </div>

      <JobProgressDrawer
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
