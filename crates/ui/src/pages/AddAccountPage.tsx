import { useRef, useState } from "react";
import { Button } from "../components/primitives/Button";
import { IconButton } from "../components/primitives/IconButton";
import { Alert } from "../components/primitives/Alert";
import { FormField } from "../components/primitives/FormField";
import { invokeCommand } from "../lib/invokeCommand";
import { save } from "@tauri-apps/plugin-dialog";
import toast from "react-hot-toast";
import {
  useCredentials,
  AwsCredentials,
  GcpCredentials,
  AzureCredentials,
} from "../hooks/useCredentials";
import { useAccounts } from "../hooks/useAccounts";
import { usePermissions } from "../hooks/usePermissions";
import { CloudProviderName, areAllPermissionsGranted } from "../types";

type VerifiableCredentials = AwsCredentials | GcpCredentials | AzureCredentials;
import { PROVIDER_METADATA } from "../constants/providers";
import { JobProgressDrawer } from "../components/common/JobProgressDrawer";
import { ProviderSelector } from "../components/providers/ProviderSelector";

interface AddAccountPageProps {
  onNavigateBack: () => void;
  onAccountAdded: () => void;
}

type AddAccountStep = "selecting-provider" | "entering-credentials";

interface SetupStep {
  number: number;
  text: string;
}

interface ProviderPolicy {
  filename: string;
  language: string;
  content: string;
}

const PROVIDER_POLICIES: Record<string, ProviderPolicy> = {
  [CloudProviderName.Aws]: {
    filename: "byocvpn-aws-policy.json",
    language: "json",
    content: JSON.stringify(
      {
        Version: "2012-10-17",
        Statement: [
          {
            Effect: "Allow",
            Action: [
              "ec2:RunInstances",
              "ec2:TerminateInstances",
              "ec2:DescribeInstances",
              "ec2:CreateVpc",
              "ec2:DeleteVpc",
              "ec2:DescribeVpcs",
              "ec2:CreateSubnet",
              "ec2:DeleteSubnet",
              "ec2:DescribeSubnets",
              "ec2:ModifySubnetAttribute",
              "ec2:CreateSecurityGroup",
              "ec2:DeleteSecurityGroup",
              "ec2:DescribeSecurityGroups",
              "ec2:AuthorizeSecurityGroupIngress",
              "ec2:CreateTags",
              "ec2:DescribeAvailabilityZones",
              "ec2:CreateInternetGateway",
              "ec2:DeleteInternetGateway",
              "ec2:AttachInternetGateway",
              "ec2:DetachInternetGateway",
              "ec2:DescribeInternetGateways",
              "ec2:CreateRoute",
              "ec2:DeleteRoute",
              "ec2:DescribeRouteTables",
              "ec2:DescribeRegions",
              "ssm:GetParameter",
            ],
            Resource: "*",
          },
        ],
      },
      null,
      2,
    ),
  },
  [CloudProviderName.Oracle]: {
    filename: "byocvpn-oci-policy.txt",
    language: "text",
    content: [
      "Allow group byocvpn-group to manage instance-family in tenancy",
      "Allow group byocvpn-group to manage virtual-cloud-networks in tenancy",
      "Allow group byocvpn-group to manage internet-gateways in tenancy",
      "Allow group byocvpn-group to manage route-tables in tenancy",
      "Allow group byocvpn-group to manage security-lists in tenancy",
      "Allow group byocvpn-group to manage subnets in tenancy",
      "Allow group byocvpn-group to manage virtual-network-interfaces in tenancy",
      "Allow group byocvpn-group to manage region-subscriptions in tenancy",
      "Allow group byocvpn-group to inspect compartments in tenancy",
      "Allow group byocvpn-group to inspect tenancies in tenancy",
      "Allow group byocvpn-group to inspect images in tenancy",
      "Allow group byocvpn-group to inspect regions in tenancy",
    ].join("\n"),
  },
  [CloudProviderName.Gcp]: {
    filename: "byocvpn-gcp-role.yaml",
    language: "yaml",
    content: [
      "title: ByocVPN",
      "description: Minimum permissions required by ByocVPN",
      "stage: GA",
      "includedPermissions:",
      "- compute.disks.create",
      "- compute.disks.delete",
      "- compute.firewalls.create",
      "- compute.firewalls.delete",
      "- compute.firewalls.get",
      "- compute.globalOperations.get",
      "- compute.images.get",
      "- compute.instances.create",
      "- compute.instances.delete",
      "- compute.instances.get",
      "- compute.instances.list",
      "- compute.instances.setLabels",
      "- compute.instances.setMetadata",
      "- compute.instances.setTags",
      "- compute.networks.create",
      "- compute.networks.delete",
      "- compute.networks.get",
      "- compute.networks.updatePolicy",
      "- compute.regions.list",
      "- compute.subnetworks.create",
      "- compute.subnetworks.delete",
      "- compute.subnetworks.get",
      "- compute.subnetworks.list",
      "- compute.subnetworks.update",
      "- compute.subnetworks.use",
      "- compute.subnetworks.useExternalIp",
      "- compute.zoneOperations.get",
      "- serviceusage.operations.get",
      "- serviceusage.services.enable",
      "- serviceusage.services.get",
    ].join("\n"),
  },
  [CloudProviderName.Azure]: {
    filename: "byocvpn-azure-role.json",
    language: "json",
    content: JSON.stringify(
      {
        properties: {
          roleName: "ByocVPN",
          description: "Minimum permissions required by ByocVPN",
          permissions: [
            {
              actions: [
                "Microsoft.Authorization/permissions/read",
                "Microsoft.Compute/register/action",
                "Microsoft.Compute/virtualMachines/read",
                "Microsoft.Compute/virtualMachines/write",
                "Microsoft.Compute/virtualMachines/delete",
                "Microsoft.Network/register/action",
                "Microsoft.Network/networkInterfaces/join/action",
                "Microsoft.Network/networkInterfaces/read",
                "Microsoft.Network/networkInterfaces/write",
                "Microsoft.Network/networkInterfaces/delete",
                "Microsoft.Network/networkSecurityGroups/join/action",
                "Microsoft.Network/networkSecurityGroups/read",
                "Microsoft.Network/networkSecurityGroups/write",
                "Microsoft.Network/networkSecurityGroups/delete",
                "Microsoft.Network/publicIPAddresses/join/action",
                "Microsoft.Network/publicIPAddresses/read",
                "Microsoft.Network/publicIPAddresses/write",
                "Microsoft.Network/publicIPAddresses/delete",
                "Microsoft.Network/virtualNetworks/read",
                "Microsoft.Network/virtualNetworks/write",
                "Microsoft.Network/virtualNetworks/delete",
                "Microsoft.Network/virtualNetworks/subnets/join/action",
                "Microsoft.Network/virtualNetworks/subnets/read",
                "Microsoft.Network/virtualNetworks/subnets/write",
                "Microsoft.Network/virtualNetworks/subnets/delete",
                "Microsoft.Resources/subscriptions/locations/read",
                "Microsoft.Resources/subscriptions/providers/read",
                "Microsoft.Resources/subscriptions/resourceGroups/read",
                "Microsoft.Resources/subscriptions/resourceGroups/write",
                "Microsoft.Resources/subscriptions/resourceGroups/delete",
              ],
              notActions: [],
              dataActions: [],
              notDataActions: [],
            },
          ],
          assignableScopes: ["/subscriptions/YOUR_SUBSCRIPTION_ID"],
        },
      },
      null,
      2,
    ),
  },
};

const PROVIDER_SETUP_INSTRUCTIONS: Record<
  string,
  { title: string; steps: SetupStep[] }
> = {
  [CloudProviderName.Aws]: {
    title: "Set up AWS credentials",
    steps: [
      {
        number: 1,
        text: "Sign in to the AWS Console and navigate to IAM → Users.",
      },
      {
        number: 2,
        text: "Click Create user and give it a name (e.g. byocvpn), then proceed to permissions.",
      },
      {
        number: 3,
        text: "For a quick setup, attach the AmazonEC2FullAccess and AmazonSSMReadOnlyAccess managed policies. For fine-grained access, choose Attach policies directly → Create policy, paste the JSON policy below into the JSON editor, save it, and attach it to the user.",
      },
      {
        number: 4,
        text: "After the user is created, open it and go to the Security credentials tab.",
      },
      {
        number: 5,
        text: "Click Create access key, select Application running outside AWS, then copy the Access Key ID and Secret Access Key.",
      },
      { number: 6, text: "Paste both values into the form and click Connect." },
    ],
  },
  [CloudProviderName.Oracle]: {
    title: "Set up Oracle Cloud credentials",
    steps: [
      { number: 1, text: "Sign in to the OCI Console at cloud.oracle.com." },
      {
        number: 2,
        text: "Click your profile icon → Tenancy to find your Tenancy OCID and home region (e.g. us-ashburn-1).",
      },
      {
        number: 3,
        text: "Click your profile icon → My profile to find your User OCID.",
      },
      {
        number: 4,
        text: "Still in My profile, open the API keys section and click Add API key. Choose Generate API key pair, download the private key (.pem file), and copy the fingerprint shown after upload.",
      },
      {
        number: 5,
        text: "If your user has administrator privileges, skip to the next step — no policies are needed. Otherwise, go to Identity & Domains → Policies, create a policy in the root compartment, and paste the policy statements below.",
      },
      {
        number: 6,
        text: "Paste your credentials and private key below, then click Connect.",
      },
    ],
  },
  [CloudProviderName.Gcp]: {
    title: "Set up Google Cloud credentials",
    steps: [
      {
        number: 1,
        text: "Sign in to the Google Cloud Console and select or create a project. Note the Project ID shown in the header.",
      },
      {
        number: 2,
        text: "Go to IAM & Admin → Service Accounts → Create service account and give it a name (e.g. byocvpn).",
      },
      {
        number: 3,
        text: "Assign permissions to the service account. Quick setup: attach the built-in roles Compute Instance Admin (v1) and Service Usage Admin. Least-privilege setup: go to IAM & Admin → Roles → Create role, add each permission listed in the YAML below, then assign that custom role to the service account.",
      },
      {
        number: 4,
        text: "Open the service account, go to the Keys tab → Add key → Create new key → JSON. A key file will download automatically.",
      },
      { number: 5, text: "Upload the JSON key file below and click Connect." },
    ],
  },
  [CloudProviderName.Azure]: {
    title: "Set up Azure credentials",
    steps: [
      {
        number: 1,
        text: "Sign in to the Azure Portal and go to Microsoft Entra ID → App registrations → New registration.",
      },
      {
        number: 2,
        text: "Register the app with any name. Note the Application (client) ID and Directory (tenant) ID shown on the overview page.",
      },
      {
        number: 3,
        text: "In the app, go to Certificates & secrets → New client secret. Copy the secret Value immediately — it won't be shown again.",
      },
      {
        number: 4,
        text: "Find your Subscription ID in the Azure Portal by searching for Subscriptions — note it down as you will need it in the next steps.",
      },
      {
        number: 5,
        text: "Create a role: quick setup — skip this step and use the built-in Contributor role in step 6. Least-privilege setup: download the JSON definition below, replace YOUR_SUBSCRIPTION_ID with your actual Subscription ID, then go to Subscriptions → your subscription → Access control (IAM) → Add → Add custom role → Start from JSON, upload the file and create the role.",
      },
      {
        number: 6,
        text: "Assign the role to your app: go to Subscriptions → your subscription → Access control (IAM) → Add → Add role assignment. Search for Contributor (quick setup) or your newly created ByocVPN role (least-privilege). Click Next, select User, group, or service principal, click Select members, search for your app registration by name, select it, then click Review + assign.",
      },
      {
        number: 7,
        text: "Enter your Subscription ID, Tenant ID, Client ID, and Client Secret in the form and click Connect.",
      },
    ],
  },
};

export function AddAccountPage({
  onNavigateBack,
  onAccountAdded,
}: AddAccountPageProps) {
  const [step, setStep] = useState<AddAccountStep>("selecting-provider");
  const [selectedProvider, setSelectedProvider] =
    useState<CloudProviderName | null>(null);

  const {
    activeProvisionJob,
    isProvisionDrawerOpen,
    isProvisionComplete,
    provisionError,
    provisionAccount,
    closeProvisionDrawer,
  } = useAccounts({
    onComplete: () => toast.success("Account connected successfully!"),
    onFailed: () => toast.error("Account setup failed"),
  });

  const { saveCredentials } = useCredentials();
  const { permissions, isVerifying, verifyPermissions, clearPermissions } =
    usePermissions();
  const [pendingCredentials, setPendingCredentials] = useState<{
    provider: CloudProviderName;
    credentials: VerifiableCredentials;
  } | null>(null);
  const [verificationFailed, setVerificationFailed] = useState(false);
  const [isVerificationDrawerOpen, setIsVerificationDrawerOpen] =
    useState(false);

  const isVerifiableProvider =
    selectedProvider === CloudProviderName.Aws ||
    selectedProvider === CloudProviderName.Gcp ||
    selectedProvider === CloudProviderName.Azure;

  const handleProviderSelected = (providerName: CloudProviderName) => {
    setSelectedProvider(providerName);
    setStep("entering-credentials");
  };

  const handleBackToProviderSelection = () => {
    setSelectedProvider(null);
    setStep("selecting-provider");
  };

  const handleCredentialsSaved = (provider: CloudProviderName) => {
    provisionAccount(provider);
  };

  const verifyAndProvision = async (
    provider: CloudProviderName,
    credentials: VerifiableCredentials,
  ) => {
    setPendingCredentials({ provider, credentials });
    setVerificationFailed(false);
    setIsVerificationDrawerOpen(true);
    const result = await verifyPermissions(provider, credentials);
    if (result && areAllPermissionsGranted(result)) {
      const saved = await saveCredentials(provider, credentials);
      if (saved) {
        provisionAccount(provider);
        return;
      }
    }
    setVerificationFailed(true);
  };

  const handleRetryVerification = () => {
    if (pendingCredentials) {
      verifyAndProvision(
        pendingCredentials.provider,
        pendingCredentials.credentials,
      );
    }
  };

  const handleCloseProvisionDrawer = () => {
    const provisioningStarted = isProvisionDrawerOpen;
    setIsVerificationDrawerOpen(false);
    setVerificationFailed(false);
    clearPermissions();
    closeProvisionDrawer();
    const hasNoSteps = (activeProvisionJob?.steps ?? []).length === 0;
    if (provisioningStarted && (isProvisionComplete || hasNoSteps)) {
      onAccountAdded();
    }
  };

  const instructions = selectedProvider
    ? PROVIDER_SETUP_INSTRUCTIONS[selectedProvider]
    : null;

  if (step === "selecting-provider") {
    return (
      <ProviderSelector
        filter="unconfigured"
        title="Add Cloud Account"
        subtitle="Connect a new cloud provider to deploy VPN servers"
        onSelectProvider={handleProviderSelected}
        onClose={onNavigateBack}
      />
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-900 text-primary">
      <div className="flex items-center gap-3 px-5 pt-5 pb-4 border-b border-gray-700/50 flex-shrink-0">
        <IconButton
          accent="white"
          size="sm"
          onClick={handleBackToProviderSelection}
          className="flex-shrink-0"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </IconButton>
        <div>
          <h1 className="text-base font-semibold text-primary leading-tight">
            {selectedProvider ? PROVIDER_METADATA[selectedProvider].label : ""}
          </h1>
          <p className="text-xs text-gray-500 mt-0.5">Enter your credentials</p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {selectedProvider && instructions && (
          <CredentialsStep
            provider={selectedProvider}
            instructions={instructions}
            onCredentialsSaved={handleCredentialsSaved}
            onVerifiableSubmit={verifyAndProvision}
            isSubmitting={isVerifying}
            onCancel={handleBackToProviderSelection}
          />
        )}
      </div>

      <JobProgressDrawer
        isOpen={isVerificationDrawerOpen || isProvisionDrawerOpen}
        onClose={handleCloseProvisionDrawer}
        provider={
          activeProvisionJob?.provider ??
          selectedProvider ??
          CloudProviderName.Aws
        }
        steps={activeProvisionJob?.steps ?? []}
        isComplete={isProvisionComplete}
        error={provisionError}
        verification={
          isVerifiableProvider
            ? { isVerifying, permissions, failed: verificationFailed }
            : undefined
        }
        onRetry={handleRetryVerification}
      />
    </div>
  );
}

interface CredentialsStepProps {
  provider: CloudProviderName;
  instructions: { title: string; steps: SetupStep[] };
  onCredentialsSaved: (provider: CloudProviderName) => void;
  onVerifiableSubmit: (
    provider: CloudProviderName,
    credentials: VerifiableCredentials,
  ) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

function CredentialsStep({
  provider,
  instructions,
  onCredentialsSaved,
  onVerifiableSubmit,
  isSubmitting,
  onCancel,
}: CredentialsStepProps) {
  const policy = PROVIDER_POLICIES[provider] ?? null;

  return (
    <div className="px-6 py-6">
      <div className="flex gap-8 items-stretch">
        <div className="max-w-xl w-full space-y-6">
          <div>
            <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-500 pb-3 border-b border-gray-700/50">
              {instructions.title}
            </h2>
            <ol className="space-y-4 pt-4">
              {instructions.steps.map((setupStep) => (
                <li key={setupStep.number} className="flex gap-3">
                  <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600/20 border border-blue-600/40 text-blue-400 text-xs font-bold flex items-center justify-center mt-0.5">
                    {setupStep.number}
                  </span>
                  <p className="text-sm text-gray-300 leading-relaxed">
                    {setupStep.text}
                  </p>
                </li>
              ))}
            </ol>
          </div>

          {provider === CloudProviderName.Aws && (
            <AwsCredentialsForm
              onSubmit={(credentials) =>
                onVerifiableSubmit(CloudProviderName.Aws, credentials)
              }
              isSubmitting={isSubmitting}
              onCancel={onCancel}
            />
          )}
          {provider === CloudProviderName.Oracle && (
            <OracleCredentialsForm
              onSaved={() => onCredentialsSaved(CloudProviderName.Oracle)}
              onCancel={onCancel}
            />
          )}
          {provider === CloudProviderName.Gcp && (
            <GcpCredentialsForm
              onSubmit={(credentials) =>
                onVerifiableSubmit(CloudProviderName.Gcp, credentials)
              }
              isSubmitting={isSubmitting}
              onCancel={onCancel}
            />
          )}
          {provider === CloudProviderName.Azure && (
            <AzureCredentialsForm
              onSubmit={(credentials) =>
                onVerifiableSubmit(CloudProviderName.Azure, credentials)
              }
              isSubmitting={isSubmitting}
              onCancel={onCancel}
            />
          )}
        </div>

        {policy && (
          <div className="flex-1 h-full">
            <PolicyBox policy={policy} />
          </div>
        )}
      </div>
    </div>
  );
}

interface ProviderFormProps {
  onSaved: () => void;
  onCancel: () => void;
}

interface AwsCredentialsFormProps {
  onSubmit: (credentials: AwsCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

function AwsCredentialsForm({
  onSubmit,
  isSubmitting,
  onCancel,
}: AwsCredentialsFormProps) {
  const [accessKeyId, setAccessKeyId] = useState("");
  const [secretAccessKey, setSecretAccessKey] = useState("");

  const handleSubmit = () => {
    onSubmit({
      accessKeyId: accessKeyId.trim(),
      secretAccessKey: secretAccessKey.trim(),
    });
  };

  const isFormValid = accessKeyId.trim() && secretAccessKey.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Access Key ID"
        hint="e.g. AKIAIOSFODNN7EXAMPLE"
        type="text"
        mono
        value={accessKeyId}
        onChange={setAccessKeyId}
      />
      <FormField
        label="Secret Access Key"
        type="password"
        value={secretAccessKey}
        onChange={setSecretAccessKey}
      />
    </CredentialsFormShell>
  );
}

function OracleCredentialsForm({ onSaved, onCancel }: ProviderFormProps) {
  const [tenancyOcid, setTenancyOcid] = useState("");
  const [userOcid, setUserOcid] = useState("");
  const [fingerprint, setFingerprint] = useState("");
  const [homeRegion, setHomeRegion] = useState("");
  const [privateKeyPem, setPrivateKeyPem] = useState("");
  const pemFileInputRef = useRef<HTMLInputElement>(null);
  const { isSaving, error, saveCredentials } = useCredentials();

  const handlePemFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const content = loadEvent.target?.result;
      if (typeof content === "string") setPrivateKeyPem(content);
    };
    reader.readAsText(file);
    event.target.value = "";
  };

  const handleSubmit = async () => {
    const success = await saveCredentials(CloudProviderName.Oracle, {
      tenancyOcid: tenancyOcid.trim(),
      userOcid: userOcid.trim(),
      fingerprint: fingerprint.trim(),
      privateKeyPem: privateKeyPem.trim(),
      region: homeRegion.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid =
    tenancyOcid.trim() &&
    userOcid.trim() &&
    fingerprint.trim() &&
    homeRegion.trim() &&
    privateKeyPem.trim();

  return (
    <CredentialsFormShell
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Tenancy OCID"
        hint="e.g. ocid1.tenancy.oc1..aaaaaa…"
        type="text"
        mono
        value={tenancyOcid}
        onChange={setTenancyOcid}
      />
      <FormField
        label="User OCID"
        hint="e.g. ocid1.user.oc1..aaaaaa…"
        type="text"
        mono
        value={userOcid}
        onChange={setUserOcid}
      />
      <FormField
        label="Key Fingerprint"
        hint="e.g. xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx"
        type="text"
        mono
        value={fingerprint}
        onChange={setFingerprint}
      />
      <FormField
        label="Home Region"
        hint="e.g. us-ashburn-1"
        type="text"
        mono
        value={homeRegion}
        onChange={setHomeRegion}
      />
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="block text-sm font-medium text-gray-300">
            Private Key (.pem)
          </label>
          <Button
            variant="secondary"
            size="none"
            type="button"
            onClick={() => pemFileInputRef.current?.click()}
            className="text-xs px-3 py-1"
          >
            Load from file
          </Button>
          <input
            ref={pemFileInputRef}
            type="file"
            accept=".pem"
            onChange={handlePemFileChange}
            className="hidden"
          />
        </div>
        <textarea
          value={privateKeyPem}
          onChange={(e) => setPrivateKeyPem(e.target.value)}
          rows={5}
          placeholder="-----BEGIN RSA PRIVATE KEY-----"
          className="input font-mono text-xs resize-none"
        />
      </div>
    </CredentialsFormShell>
  );
}

interface GcpCredentialsFormProps {
  onSubmit: (credentials: GcpCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

function GcpCredentialsForm({
  onSubmit,
  isSubmitting,
  onCancel,
}: GcpCredentialsFormProps) {
  const [projectId, setProjectId] = useState("");
  const [serviceAccountJson, setServiceAccountJson] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const content = loadEvent.target?.result;
      if (typeof content === "string") {
        setServiceAccountJson(content);
        try {
          const parsed = JSON.parse(content);
          if (parsed.project_id && !projectId) setProjectId(parsed.project_id);
        } catch {}
      }
    };
    reader.readAsText(file);
    event.target.value = "";
  };

  const handleSubmit = () => {
    onSubmit({
      projectId: projectId.trim(),
      serviceAccountJson: serviceAccountJson.trim(),
    });
  };

  const isFormValid = projectId.trim() && serviceAccountJson.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Project ID"
        hint="e.g. my-project-123456"
        type="text"
        mono
        value={projectId}
        onChange={setProjectId}
      />
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="block text-sm font-medium text-gray-300">
            Service Account Key (.json)
          </label>
          <Button
            variant="secondary"
            size="none"
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="text-xs px-3 py-1"
          >
            Load from file
          </Button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".json,application/json"
            onChange={handleFileChange}
            className="hidden"
          />
        </div>
        <textarea
          value={serviceAccountJson}
          onChange={(e) => setServiceAccountJson(e.target.value)}
          rows={6}
          placeholder='{"type":"service_account","project_id":"..."}'
          className="input font-mono text-xs resize-none"
        />
      </div>
    </CredentialsFormShell>
  );
}

interface AzureCredentialsFormProps {
  onSubmit: (credentials: AzureCredentials) => void;
  isSubmitting: boolean;
  onCancel: () => void;
}

function AzureCredentialsForm({
  onSubmit,
  isSubmitting,
  onCancel,
}: AzureCredentialsFormProps) {
  const [subscriptionId, setSubscriptionId] = useState("");
  const [tenantId, setTenantId] = useState("");
  const [applicationId, setApplicationId] = useState("");
  const [secretValue, setSecretValue] = useState("");

  const handleSubmit = () => {
    onSubmit({
      subscriptionId: subscriptionId.trim(),
      tenantId: tenantId.trim(),
      applicationId: applicationId.trim(),
      secretValue: secretValue.trim(),
    });
  };

  const isFormValid =
    subscriptionId.trim() &&
    tenantId.trim() &&
    applicationId.trim() &&
    secretValue.trim();

  return (
    <CredentialsFormShell
      error={null}
      isSaving={isSubmitting}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <FormField
        label="Subscription ID"
        hint="e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
        type="text"
        mono
        value={subscriptionId}
        onChange={setSubscriptionId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Tenant ID"
        hint="Your Azure Active Directory tenant ID"
        type="text"
        mono
        value={tenantId}
        onChange={setTenantId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Application ID"
        hint="Application ID of the service principal"
        type="text"
        mono
        value={applicationId}
        onChange={setApplicationId}
        placeholder="00000000-0000-0000-0000-000000000000"
      />
      <FormField
        label="Secret Value"
        hint="Secret value from your app registration"
        type="password"
        mono
        value={secretValue}
        onChange={setSecretValue}
      />
    </CredentialsFormShell>
  );
}

interface PolicyBoxProps {
  policy: ProviderPolicy;
}

function PolicyBox({ policy }: PolicyBoxProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(policy.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = async () => {
    const savePath = await save({ defaultPath: policy.filename });
    if (savePath) {
      await invokeCommand("save_file", {
        path: savePath,
        content: policy.content,
      });
    }
  };

  return (
    <div className="bg-gray-800 rounded-xl overflow-hidden h-full flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700/50">
        <div className="flex items-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-4 w-4 text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <span className="text-sm text-gray-300 font-mono">
            {policy.filename}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="none"
            onClick={handleCopy}
            className="px-3 py-1.5 text-xs"
          >
            {copied ? (
              <>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-3.5 w-3.5 text-success-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Copied
              </>
            ) : (
              <>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-3.5 w-3.5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                  />
                </svg>
                Copy
              </>
            )}
          </Button>
          <Button
            variant="secondary"
            size="none"
            onClick={handleDownload}
            className="px-3 py-1.5 text-xs"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-3.5 w-3.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
              />
            </svg>
            Download
          </Button>
        </div>
      </div>
      <pre className="p-4 text-xs font-mono text-gray-300 overflow-x-auto flex-1 overflow-y-auto leading-relaxed">
        {policy.content}
      </pre>
    </div>
  );
}

interface CredentialsFormShellProps {
  error: string | null;
  isSaving: boolean;
  isFormValid: boolean;
  onSubmit: () => void;
  onCancel: () => void;
  children: React.ReactNode;
}

function CredentialsFormShell({
  error,
  isSaving,
  isFormValid,
  onSubmit,
  onCancel,
  children,
}: CredentialsFormShellProps) {
  return (
    <div>
      <div className="space-y-4">
        {children}

        {error && <Alert variant="error">{error}</Alert>}

        <div className="flex gap-3 pt-2">
          <Button variant="secondary" onClick={onCancel} className="flex-1">
            Back
          </Button>
          <Button
            variant="primary"
            onClick={onSubmit}
            loading={isSaving}
            disabled={!isFormValid}
            className="flex-1"
          >
            {isSaving ? "Saving..." : "Connect"}
          </Button>
        </div>
      </div>
    </div>
  );
}
