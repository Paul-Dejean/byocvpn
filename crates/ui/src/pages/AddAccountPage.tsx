import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import toast from "react-hot-toast";
import { useCredentials } from "../hooks/useCredentials";
import { ProvisionAccountDrawer } from "../components/settings/ProvisionAccountDrawer";
import {
  ProvisionAccountJob,
  ProvisionAccountProgressEvent,
  ProvisionAccountCompleteEvent,
  ProvisionJobState,
  SpawnStepState,
} from "../types";

interface AddAccountPageProps {
  onNavigateBack: () => void;
  onAccountAdded: () => void;
}

type AddAccountStep = "selecting-provider" | "entering-credentials";

interface ProviderOption {
  id: string;
  label: string;
  description: string;
  badge: React.ReactNode;
}

const ALL_PROVIDERS: ProviderOption[] = [
  {
    id: "aws",
    label: "Amazon Web Services",
    description: "Deploy on EC2 — available in 30+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-orange-500/20 flex items-center justify-center flex-shrink-0">
        <span className="text-orange-400 font-black text-xl">AWS</span>
      </div>
    ),
  },
  {
    id: "oracle",
    label: "Oracle Cloud Infrastructure",
    description: "Deploy on OCI Compute — includes an Always Free tier",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-red-700/30 flex items-center justify-center flex-shrink-0">
        <span className="text-red-400 font-black text-xl">OCI</span>
      </div>
    ),
  },
  {
    id: "gcp",
    label: "Google Cloud Platform",
    description: "Deploy on Compute Engine using a service account — available in 40+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-blue-600/20 flex items-center justify-center flex-shrink-0">
        <span className="text-blue-400 font-black text-xl">GCP</span>
      </div>
    ),
  },
  {
    id: "azure",
    label: "Microsoft Azure",
    description: "Deploy on Azure VMs — available in 60+ regions worldwide",
    badge: (
      <div className="w-14 h-14 rounded-xl bg-sky-600/20 flex items-center justify-center flex-shrink-0">
        <span className="text-sky-400 font-black text-xl">AZ</span>
      </div>
    ),
  },
];

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
  aws: {
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
  oracle: {
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
  gcp: {
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
  azure: {
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

const PROVIDER_SETUP_INSTRUCTIONS: Record<string, { title: string; steps: SetupStep[] }> = {
  aws: {
    title: "Set up AWS credentials",
    steps: [
      { number: 1, text: "Sign in to the AWS Console and navigate to IAM → Users." },
      { number: 2, text: "Click Create user and give it a name (e.g. byocvpn), then proceed to permissions." },
      { number: 3, text: "For a quick setup, attach the AmazonEC2FullAccess and AmazonSSMReadOnlyAccess managed policies. For fine-grained access, choose Attach policies directly → Create policy, paste the JSON policy below into the JSON editor, save it, and attach it to the user." },
      { number: 4, text: "After the user is created, open it and go to the Security credentials tab." },
      { number: 5, text: "Click Create access key, select Application running outside AWS, then copy the Access Key ID and Secret Access Key." },
      { number: 6, text: "Paste both values into the form and click Connect." },
    ],
  },
  oracle: {
    title: "Set up Oracle Cloud credentials",
    steps: [
      { number: 1, text: "Sign in to the OCI Console at cloud.oracle.com." },
      { number: 2, text: "Click your profile icon → Tenancy to find your Tenancy OCID and home region (e.g. us-ashburn-1)." },
      { number: 3, text: "Click your profile icon → My profile to find your User OCID." },
      { number: 4, text: "Still in My profile, open the API keys section and click Add API key. Choose Generate API key pair, download the private key (.pem file), and copy the fingerprint shown after upload." },
      { number: 5, text: "If your user has administrator privileges, skip to the next step — no policies are needed. Otherwise, go to Identity & Domains → Policies, create a policy in the root compartment, and paste the policy statements below." },
      { number: 6, text: "Paste your credentials and private key below, then click Connect." },
    ],
  },
  gcp: {
    title: "Set up Google Cloud credentials",
    steps: [
      { number: 1, text: "Sign in to the Google Cloud Console and select or create a project. Note the Project ID shown in the header." },
      { number: 2, text: "Go to IAM & Admin → Service Accounts → Create service account and give it a name (e.g. byocvpn)." },
      { number: 3, text: "Assign permissions to the service account. Quick setup: attach the built-in roles Compute Instance Admin (v1) and Service Usage Admin. Least-privilege setup: go to IAM & Admin → Roles → Create role, add each permission listed in the YAML below, then assign that custom role to the service account." },
      { number: 4, text: "Open the service account, go to the Keys tab → Add key → Create new key → JSON. A key file will download automatically." },
      { number: 5, text: "Upload the JSON key file below and click Connect." },
    ],
  },
  azure: {
    title: "Set up Azure credentials",
    steps: [
      { number: 1, text: "Sign in to the Azure Portal and go to Microsoft Entra ID → App registrations → New registration." },
      { number: 2, text: "Register the app with any name. Note the Application (client) ID and Directory (tenant) ID shown on the overview page." },
      { number: 3, text: "In the app, go to Certificates & secrets → New client secret. Copy the secret Value immediately — it won't be shown again." },
      { number: 4, text: "Find your Subscription ID in the Azure Portal by searching for Subscriptions — note it down as you will need it in the next steps." },
      { number: 5, text: "Create a role: quick setup — skip this step and use the built-in Contributor role in step 6. Least-privilege setup: download the JSON definition below, replace YOUR_SUBSCRIPTION_ID with your actual Subscription ID, then go to Subscriptions → your subscription → Access control (IAM) → Add → Add custom role → Start from JSON, upload the file and create the role." },
      { number: 6, text: "Assign the role to your app: go to Subscriptions → your subscription → Access control (IAM) → Add → Add role assignment. Search for Contributor (quick setup) or your newly created ByocVPN role (least-privilege). Click Next, select User, group, or service principal, click Select members, search for your app registration by name, select it, then click Review + assign." },
      { number: 7, text: "Enter your Subscription ID, Tenant ID, Client ID, and Client Secret in the form and click Connect." },
    ],
  },
};

export function AddAccountPage({ onNavigateBack, onAccountAdded }: AddAccountPageProps) {
  const [step, setStep] = useState<AddAccountStep>("selecting-provider");
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [unconfiguredProviders, setUnconfiguredProviders] = useState<ProviderOption[]>([]);
  const [isLoadingProviders, setIsLoadingProviders] = useState(true);

  const [activeProvisionJob, setActiveProvisionJob] = useState<ProvisionJobState | null>(null);
  const [isProvisionDrawerOpen, setIsProvisionDrawerOpen] = useState(false);
  const [isProvisionComplete, setIsProvisionComplete] = useState(false);
  const [provisionError, setProvisionError] = useState<string | null>(null);
  const activeJobIdRef = useRef<string | null>(null);
  const earlyProgressEventsRef = useRef<ProvisionAccountProgressEvent[]>([]);

  const { loadCredentials } = useCredentials();

  useEffect(() => {
    const loadUnconfiguredProviders = async () => {
      const unconfigured: ProviderOption[] = [];
      for (const provider of ALL_PROVIDERS) {
        const existing = await loadCredentials(provider.id as "aws" | "oracle" | "gcp" | "azure");
        if (existing === null) {
          unconfigured.push(provider);
        }
      }
      setUnconfiguredProviders(unconfigured);
      setIsLoadingProviders(false);
    };
    loadUnconfiguredProviders();
  }, []);

  useEffect(() => {
    const progressUnlisten = listen<ProvisionAccountProgressEvent>(
      "provision-account-progress",
      ({ payload }) => {
        const { jobId, stepId, status, error: stepError } = payload;
        setActiveProvisionJob((previous) => {
          if (!previous || previous.jobId !== jobId) {
            earlyProgressEventsRef.current.push(payload);
            return previous;
          }
          return {
            ...previous,
            steps: previous.steps.map((provisionStep) =>
              provisionStep.id === stepId
                ? { ...provisionStep, status, error: stepError }
                : provisionStep,
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
          toast.success("Account connected successfully!");
        }
      },
    );

    const failedUnlisten = listen<{ jobId: string; error: string }>(
      "provision-account-failed",
      ({ payload }) => {
        if (activeJobIdRef.current === payload.jobId) {
          setProvisionError(payload.error);
          toast.error("Account setup failed");
        }
      },
    );

    return () => {
      progressUnlisten.then((unlisten) => unlisten());
      completeUnlisten.then((unlisten) => unlisten());
      failedUnlisten.then((unlisten) => unlisten());
    };
  }, []);

  const startProvisioning = async (provider: string) => {
    try {
      earlyProgressEventsRef.current = [];
      const job = await invoke<ProvisionAccountJob>("provision_account", { provider });
      const bufferedEvents = earlyProgressEventsRef.current.filter((event) => event.jobId === job.jobId);
      earlyProgressEventsRef.current = [];
      const initialSteps: SpawnStepState[] = job.steps.map((provisionStep) => {
        const latestBufferedEvent = [...bufferedEvents].reverse().find((event) => event.stepId === provisionStep.id);
        return {
          ...provisionStep,
          status: latestBufferedEvent?.status ?? ("pending" as const),
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
        invocationError instanceof Error ? invocationError.message : "Failed to start setup";
      toast.error(message);
    }
  };

  const handleProviderSelected = (providerId: string) => {
    setSelectedProvider(providerId);
    setStep("entering-credentials");
  };

  const handleBackToProviderSelection = () => {
    setSelectedProvider(null);
    setStep("selecting-provider");
  };

  const handleCredentialsSaved = (provider: string) => {
    startProvisioning(provider);
  };

  const handleCloseProvisionDrawer = () => {
    setIsProvisionDrawerOpen(false);
    const hasNoSteps = (activeProvisionJob?.steps ?? []).length === 0;
    if (isProvisionComplete || hasNoSteps) {
      onAccountAdded();
    }
  };

  const selectedProviderOption = ALL_PROVIDERS.find((p) => p.id === selectedProvider) ?? null;
  const instructions = selectedProvider ? PROVIDER_SETUP_INSTRUCTIONS[selectedProvider] : null;

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
      <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-700/40 flex-shrink-0">
        <button
          onClick={step === "entering-credentials" ? handleBackToProviderSelection : onNavigateBack}
          className="p-1.5 rounded-lg text-gray-500 hover:text-gray-200 hover:bg-gray-700/50 transition-colors"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-4 w-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <span className="text-sm font-semibold text-gray-300">
          {step === "selecting-provider" ? "Connect a Cloud Account" : `Connect ${selectedProviderOption?.label ?? ""}`}
        </span>
      </div>

      <div className="flex-1 overflow-y-auto">
        {step === "selecting-provider" ? (
          <ProviderSelectionStep
            providers={unconfiguredProviders}
            isLoading={isLoadingProviders}
            onSelectProvider={handleProviderSelected}
          />
        ) : (
          selectedProvider && instructions && (
            <CredentialsStep
              provider={selectedProvider}
              instructions={instructions}
              onCredentialsSaved={handleCredentialsSaved}
              onCancel={handleBackToProviderSelection}
            />
          )
        )}
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

interface ProviderSelectionStepProps {
  providers: ProviderOption[];
  isLoading: boolean;
  onSelectProvider: (providerId: string) => void;
}

function ProviderSelectionStep({ providers, isLoading, onSelectProvider }: ProviderSelectionStepProps) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-6 h-6 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (providers.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 text-gray-400">
        <p className="text-lg font-medium">All providers are already connected.</p>
        <p className="text-sm">Manage existing accounts from Settings.</p>
      </div>
    );
  }

  return (
    <div className="p-6 flex flex-col gap-4 max-w-lg mx-auto w-full">
      {providers.map((provider) => (
        <button
          key={provider.id}
          onClick={() => onSelectProvider(provider.id)}
          className="w-full flex items-center gap-5 p-5 bg-gray-800 card-border rounded-xl hover:glow-accent-sm transition-all text-left group"
        >
          {provider.badge}
          <div className="flex-1">
            <p className="font-semibold text-white text-lg group-hover:text-blue-300 transition-colors">
              {provider.label}
            </p>
            <p className="text-sm text-gray-400 mt-0.5">{provider.description}</p>
          </div>
          <svg
            className="w-5 h-5 text-gray-500 group-hover:text-blue-400 transition-colors flex-shrink-0"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>
      ))}
    </div>
  );
}

interface CredentialsStepProps {
  provider: string;
  instructions: { title: string; steps: SetupStep[] };
  onCredentialsSaved: (provider: string) => void;
  onCancel: () => void;
}

function CredentialsStep({ provider, instructions, onCredentialsSaved, onCancel }: CredentialsStepProps) {
  const policy = PROVIDER_POLICIES[provider] ?? null;

  return (
    <div className="p-6 max-w-5xl mx-auto w-full">
      <div className="flex gap-8 items-start">
        <div className="flex-1 min-w-0 space-y-4">
          <div className="bg-gray-800 rounded-xl p-6">
            <h2 className="text-lg font-semibold text-white mb-4">{instructions.title}</h2>
            <ol className="space-y-4">
              {instructions.steps.map((setupStep) => (
                <li key={setupStep.number} className="flex gap-3">
                  <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600/20 border border-blue-600/40 text-blue-400 text-xs font-bold flex items-center justify-center mt-0.5">
                    {setupStep.number}
                  </span>
                  <p className="text-sm text-gray-300 leading-relaxed">{setupStep.text}</p>
                </li>
              ))}
            </ol>
          </div>
          {policy && <PolicyBox policy={policy} />}
        </div>

        <div className="w-96 flex-shrink-0">
          {provider === "aws" && (
            <AwsCredentialsForm onSaved={() => onCredentialsSaved("aws")} onCancel={onCancel} />
          )}
          {provider === "oracle" && (
            <OracleCredentialsForm onSaved={() => onCredentialsSaved("oracle")} onCancel={onCancel} />
          )}
          {provider === "gcp" && (
            <GcpCredentialsForm onSaved={() => onCredentialsSaved("gcp")} onCancel={onCancel} />
          )}
          {provider === "azure" && (
            <AzureCredentialsForm onSaved={() => onCredentialsSaved("azure")} onCancel={onCancel} />
          )}
        </div>
      </div>
    </div>
  );
}

interface ProviderFormProps {
  onSaved: () => void;
  onCancel: () => void;
}

function AwsCredentialsForm({ onSaved, onCancel }: ProviderFormProps) {
  const [accessKeyId, setAccessKeyId] = useState("");
  const [secretAccessKey, setSecretAccessKey] = useState("");
  const { isSaving, error, saveCredentials } = useCredentials();

  const handleSubmit = async () => {
    const success = await saveCredentials("aws", {
      accessKeyId: accessKeyId.trim(),
      secretAccessKey: secretAccessKey.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid = accessKeyId.trim() && secretAccessKey.trim();

  return (
    <CredentialsFormShell
      title="AWS credentials"
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Access Key ID</label>
        <p className="text-xs text-gray-500 mb-2">e.g. AKIAIOSFODNN7EXAMPLE</p>
        <input
          type="text"
          value={accessKeyId}
          onChange={(e) => setAccessKeyId(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Secret Access Key</label>
        <input
          type="password"
          value={secretAccessKey}
          onChange={(e) => setSecretAccessKey(e.target.value)}
          className="input"
        />
      </div>
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
    const success = await saveCredentials("oracle", {
      tenancyOcid: tenancyOcid.trim(),
      userOcid: userOcid.trim(),
      fingerprint: fingerprint.trim(),
      privateKeyPem: privateKeyPem.trim(),
      region: homeRegion.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid =
    tenancyOcid.trim() && userOcid.trim() && fingerprint.trim() && homeRegion.trim() && privateKeyPem.trim();

  return (
    <CredentialsFormShell
      title="Oracle Cloud credentials"
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Tenancy OCID</label>
        <p className="text-xs text-gray-500 mb-2">e.g. ocid1.tenancy.oc1..aaaaaa…</p>
        <input
          type="text"
          value={tenancyOcid}
          onChange={(e) => setTenancyOcid(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">User OCID</label>
        <p className="text-xs text-gray-500 mb-2">e.g. ocid1.user.oc1..aaaaaa…</p>
        <input
          type="text"
          value={userOcid}
          onChange={(e) => setUserOcid(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Key Fingerprint</label>
        <p className="text-xs text-gray-500 mb-2">e.g. xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx:xx</p>
        <input
          type="text"
          value={fingerprint}
          onChange={(e) => setFingerprint(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Home Region</label>
        <p className="text-xs text-gray-500 mb-2">e.g. us-ashburn-1</p>
        <input
          type="text"
          value={homeRegion}
          onChange={(e) => setHomeRegion(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="block text-sm font-medium text-gray-300">Private Key (.pem)</label>
          <button
            type="button"
            onClick={() => pemFileInputRef.current?.click()}
            className="text-xs px-3 py-1 bg-gray-600 hover:bg-gray-500 text-gray-300 rounded-lg transition-colors"
          >
            Load from file
          </button>
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

function GcpCredentialsForm({ onSaved, onCancel }: ProviderFormProps) {
  const [projectId, setProjectId] = useState("");
  const [serviceAccountJson, setServiceAccountJson] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { isSaving, error, saveCredentials } = useCredentials();

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
        } catch {
        }
      }
    };
    reader.readAsText(file);
    event.target.value = "";
  };

  const handleSubmit = async () => {
    const success = await saveCredentials("gcp", {
      projectId: projectId.trim(),
      serviceAccountJson: serviceAccountJson.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid = projectId.trim() && serviceAccountJson.trim();

  return (
    <CredentialsFormShell
      title="Google Cloud credentials"
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Project ID</label>
        <p className="text-xs text-gray-500 mb-2">e.g. my-project-123456</p>
        <input
          type="text"
          value={projectId}
          onChange={(e) => setProjectId(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="block text-sm font-medium text-gray-300">Service Account Key (.json)</label>
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="text-xs px-3 py-1 bg-gray-600 hover:bg-gray-500 text-gray-300 rounded-lg transition-colors"
          >
            Load from file
          </button>
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

function AzureCredentialsForm({ onSaved, onCancel }: ProviderFormProps) {
  const [subscriptionId, setSubscriptionId] = useState("");
  const [tenantId, setTenantId] = useState("");
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const { isSaving, error, saveCredentials } = useCredentials();

  const handleSubmit = async () => {
    const success = await saveCredentials("azure", {
      subscriptionId: subscriptionId.trim(),
      tenantId: tenantId.trim(),
      clientId: clientId.trim(),
      clientSecret: clientSecret.trim(),
    });
    if (success) onSaved();
  };

  const isFormValid =
    subscriptionId.trim() && tenantId.trim() && clientId.trim() && clientSecret.trim();

  return (
    <CredentialsFormShell
      title="Azure credentials"
      error={error}
      isSaving={isSaving}
      isFormValid={!!isFormValid}
      onSubmit={handleSubmit}
      onCancel={onCancel}
    >
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Subscription ID</label>
        <p className="text-xs text-gray-500 mb-2">e.g. xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx</p>
        <input
          type="text"
          value={subscriptionId}
          onChange={(e) => setSubscriptionId(e.target.value)}
          className="input font-mono text-sm"
          placeholder="00000000-0000-0000-0000-000000000000"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Tenant ID</label>
        <p className="text-xs text-gray-500 mb-2">Your Azure Active Directory tenant ID</p>
        <input
          type="text"
          value={tenantId}
          onChange={(e) => setTenantId(e.target.value)}
          className="input font-mono text-sm"
          placeholder="00000000-0000-0000-0000-000000000000"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Client ID</label>
        <p className="text-xs text-gray-500 mb-2">Application (client) ID of the service principal</p>
        <input
          type="text"
          value={clientId}
          onChange={(e) => setClientId(e.target.value)}
          className="input font-mono text-sm"
          placeholder="00000000-0000-0000-0000-000000000000"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">Client Secret</label>
        <p className="text-xs text-gray-500 mb-2">Secret value from your app registration</p>
        <input
          type="password"
          value={clientSecret}
          onChange={(e) => setClientSecret(e.target.value)}
          className="input font-mono text-sm"
        />
      </div>
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
      await invoke("save_file", { path: savePath, content: policy.content });
    }
  };

  return (
    <div className="bg-gray-800 rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700/50">
        <div className="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <span className="text-sm text-gray-300 font-mono">{policy.filename}</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white rounded-lg transition-colors"
          >
            {copied ? (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" className="h-3.5 w-3.5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Copied
              </>
            ) : (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
                Copy
              </>
            )}
          </button>
          <button
            onClick={handleDownload}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white rounded-lg transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Download
          </button>
        </div>
      </div>
      <pre className="p-4 text-xs font-mono text-gray-300 overflow-x-auto max-h-48 overflow-y-auto leading-relaxed">
        {policy.content}
      </pre>
    </div>
  );
}

interface CredentialsFormShellProps {
  title: string;
  error: string | null;
  isSaving: boolean;
  isFormValid: boolean;
  onSubmit: () => void;
  onCancel: () => void;
  children: React.ReactNode;
}

function CredentialsFormShell({
  title,
  error,
  isSaving,
  isFormValid,
  onSubmit,
  onCancel,
  children,
}: CredentialsFormShellProps) {
  return (
    <div className="bg-gray-800 rounded-xl p-6">
      <h2 className="text-lg font-semibold text-white mb-5">{title}</h2>
      <div className="space-y-4">
        {children}

        {error && (
          <div className="p-3 bg-red-900 border border-red-700 rounded-lg">
            <p className="text-red-300 text-sm">{error}</p>
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-2 btn-secondary"
          >
            Back
          </button>
          <button
            onClick={onSubmit}
            disabled={isSaving || !isFormValid}
            className="btn-primary flex-1 px-4 py-2 disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed disabled:hover:bg-gray-600"
          >
            {isSaving ? (
              <div className="flex items-center justify-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                Saving...
              </div>
            ) : (
              "Connect"
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
