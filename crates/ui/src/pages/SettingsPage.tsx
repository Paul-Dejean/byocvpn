import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
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
import { PermissionsDrawer } from "../components/common/PermissionsDrawer";
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

  const { loadCredentials } = useCredentials();

  const {
    activeProvisionJob,
    isProvisionComplete,
    provisionError,
    provisionAccount,
    resetProvisionState,
  } = useAccounts({
    onFailed: () => toast.error("Account setup failed"),
  });

  const {
    permissions,
    isVerifying,
    error: permissionsError,
    verifyPermissions,
    clearPermissions,
  } = usePermissions();
  const [verifiedProvider, setVerifiedProvider] = useState<CloudProviderName | null>(null);
  const isProvisionDrawerOpen = activeProvisionJob !== null;

  function handleVerifyRequested(provider: CloudProviderName) {
    setVerifiedProvider(provider);
    verifyPermissions(provider);
  }

  function handleClosePermissionsDrawer() {
    setVerifiedProvider(null);
    clearPermissions();
  }

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

  const hasConfiguredAccount =
    awsHasCredentials || oracleHasCredentials || gcpHasCredentials || azureHasCredentials;
  const canAddAccount =
    onNavigateToAddAccount !== undefined &&
    !(awsHasCredentials && oracleHasCredentials && gcpHasCredentials && azureHasCredentials);

  return (
    <div className="flex flex-col h-full gap-3">
      <header className="flex flex-col gap-1">
        <h1 className="text-body-sm font-medium text-fg-lighter">Settings</h1>
        <p className="text-caption text-fg-medium">
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
              onVerifyRequested={handleVerifyRequested}
              onCredentialsDeleted={() => {
                setAwsHasCredentials(false);
              }}
            />
          )}
          {oracleHasCredentials === true && (
            <OracleAccountCard
              onCredentialsSaved={provisionAccount}
              onCredentialsDeleted={() => {
                setOracleHasCredentials(false);
              }}
            />
          )}
          {gcpHasCredentials === true && (
            <GcpAccountCard
              onCredentialsSaved={provisionAccount}
              onVerifyRequested={handleVerifyRequested}
              onCredentialsDeleted={() => {
                setGcpHasCredentials(false);
              }}
            />
          )}
          {azureHasCredentials === true && (
            <AzureAccountCard
              onCredentialsSaved={provisionAccount}
              onVerifyRequested={handleVerifyRequested}
              onCredentialsDeleted={() => {
                setAzureHasCredentials(false);
              }}
            />
          )}
          {!hasConfiguredAccount && (
            <p className="p-4 text-caption text-fg-medium">No cloud account connected yet.</p>
          )}
          {canAddAccount && (
            <div className="p-4">
              <Button
                variant="secondary"
                size="lg"
                onClick={onNavigateToAddAccount}
                icon={<Plus size={14} />}
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
      />

      <PermissionsDrawer
        provider={verifiedProvider}
        permissions={permissions}
        isVerifying={isVerifying}
        error={permissionsError}
        onClose={handleClosePermissionsDrawer}
        onRetry={() => {
          if (verifiedProvider) {
            verifyPermissions(verifiedProvider);
          }
        }}
      />
    </div>
  );
}
