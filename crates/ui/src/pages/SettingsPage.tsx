import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { load as loadStore } from "@tauri-apps/plugin-store";
import toast from "react-hot-toast";
import { useCredentials } from "../hooks/useCredentials";
import { useAccounts } from "../hooks/useAccounts";
import { usePermissions } from "../hooks/usePermissions";
import { CloudProviderName } from "../types";
import { AwsAccountCard } from "../components/settings/AwsAccountCard";
import { OracleAccountCard } from "../components/settings/OracleAccountCard";
import { GcpAccountCard } from "../components/settings/GcpAccountCard";
import { AzureAccountCard } from "../components/settings/AzureAccountCard";
import { JobProgressDrawer } from "../components/common/JobProgressDrawer";
import { NotificationSettingsCard } from "../components/settings/NotificationSettingsCard";
import { SessionKillswitchCard } from "../components/settings/SessionKillswitchCard";
import { AutoTerminateSettingsCard } from "../components/settings/AutoTerminateSettingsCard";
import { AppearanceCard } from "../components/settings/AppearanceCard";
import { SettingsSection } from "../components/settings/SettingsSection";
import { Button } from "../components/primitives/Button";

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
    isProvisionComplete,
    provisionError,
    provisionAccount,
    resetProvisionState,
  } = useAccounts({
    onComplete: (provider) => {
      setProvisionedProviders((previous) => new Set([...previous, provider]));
    },
    onFailed: () => toast.error("Provisioning failed"),
  });

  const { permissions, isVerifying, verifyPermissions, clearPermissions } =
    usePermissions();
  const isProvisionDrawerOpen = activeProvisionJob !== null;
  const provisionJobProvider = activeProvisionJob?.provider;
  const isVerifiableProvisionJob =
    provisionJobProvider === CloudProviderName.Aws ||
    provisionJobProvider === CloudProviderName.Gcp ||
    provisionJobProvider === CloudProviderName.Azure;

  useEffect(() => {
    if (isVerifiableProvisionJob && provisionJobProvider) {
      verifyPermissions(provisionJobProvider);
    }
    if (!provisionJobProvider) {
      clearPermissions();
    }
  }, [
    isVerifiableProvisionJob,
    provisionJobProvider,
    verifyPermissions,
    clearPermissions,
  ]);

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

  function removeProvisionedProvider(provider: CloudProviderName) {
    setProvisionedProviders((previous) => {
      const next = new Set(previous);
      next.delete(provider);
      return next;
    });
  }

  const hasConfiguredAccount =
    awsHasCredentials || oracleHasCredentials || gcpHasCredentials || azureHasCredentials;
  const canAddAccount =
    onNavigateToAddAccount !== undefined &&
    !(awsHasCredentials && oracleHasCredentials && gcpHasCredentials && azureHasCredentials);

  return (
    <div className="flex flex-col h-full gap-3">
      <header className="flex flex-col gap-1">
        <h1 className="text-sm font-medium text-primary">Settings</h1>
        <p className="text-xs text-gray-300">
          Manage your cloud accounts and how ByocVPN behaves.
        </p>
      </header>

      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-6 pr-1">
        <SettingsSection
          title="Cloud accounts"
          description="Credentials stay on your device. We never store them."
        >
          {awsHasCredentials === true && (
            <AwsAccountCard
              onCredentialsSaved={provisionAccount}
              onProvisionRequested={provisionAccount}
              isProvisioned={provisionedProviders.has(CloudProviderName.Aws)}
              onCredentialsDeleted={() => {
                setAwsHasCredentials(false);
                removeProvisionedProvider(CloudProviderName.Aws);
              }}
            />
          )}
          {oracleHasCredentials === true && (
            <OracleAccountCard
              onCredentialsSaved={provisionAccount}
              onProvisionRequested={provisionAccount}
              isProvisioned={provisionedProviders.has(CloudProviderName.Oracle)}
              onCredentialsDeleted={() => {
                setOracleHasCredentials(false);
                removeProvisionedProvider(CloudProviderName.Oracle);
              }}
            />
          )}
          {gcpHasCredentials === true && (
            <GcpAccountCard
              onCredentialsSaved={provisionAccount}
              onProvisionRequested={provisionAccount}
              isProvisioned={provisionedProviders.has(CloudProviderName.Gcp)}
              onCredentialsDeleted={() => {
                setGcpHasCredentials(false);
                removeProvisionedProvider(CloudProviderName.Gcp);
              }}
            />
          )}
          {azureHasCredentials === true && (
            <AzureAccountCard
              onCredentialsSaved={provisionAccount}
              onProvisionRequested={provisionAccount}
              isProvisioned={provisionedProviders.has(CloudProviderName.Azure)}
              onCredentialsDeleted={() => {
                setAzureHasCredentials(false);
                removeProvisionedProvider(CloudProviderName.Azure);
              }}
            />
          )}
          {!hasConfiguredAccount && (
            <p className="p-4 text-xs text-gray-300">No cloud account connected yet.</p>
          )}
          {canAddAccount && (
            <div className="p-4">
              <Button
                variant="secondary"
                size="none"
                onClick={onNavigateToAddAccount}
                icon={<Plus size={14} />}
                className="px-3 py-1.5 text-sm"
              >
                Add account
              </Button>
            </div>
          )}
        </SettingsSection>

        <SettingsSection title="Appearance">
          <AppearanceCard />
        </SettingsSection>

        <SettingsSection title="VPN">
          <SessionKillswitchCard />
          <NotificationSettingsCard />
          <AutoTerminateSettingsCard />
        </SettingsSection>
      </div>

      <JobProgressDrawer
        isOpen={isProvisionDrawerOpen}
        onClose={resetProvisionState}
        provider={activeProvisionJob?.provider ?? CloudProviderName.Aws}
        steps={activeProvisionJob?.steps ?? []}
        isComplete={isProvisionComplete}
        error={provisionError}
        verification={
          isVerifiableProvisionJob
            ? { isVerifying, permissions, failed: false }
            : undefined
        }
      />
    </div>
  );
}
