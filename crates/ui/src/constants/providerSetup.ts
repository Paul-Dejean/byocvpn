import { CloudProviderName } from "../types";

export interface SetupStep {
  number: number;
  text: string;
}

export interface ProviderSetupInstructions {
  title: string;
  steps: SetupStep[];
}

export interface ProviderPolicy {
  filename: string;
  language: string;
  content: string;
}

export const PROVIDER_POLICIES: Record<string, ProviderPolicy> = {
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
              "ec2:RevokeSecurityGroupIngress",
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
          {
            Effect: "Allow",
            Action: "iam:SimulatePrincipalPolicy",
            Resource: "arn:aws:iam::*:user/${aws:username}",
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
      "- compute.firewalls.update",
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
      "- compute.regionOperations.get",
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
                "Microsoft.Compute/locations/operations/read",
                "Microsoft.Compute/virtualMachines/read",
                "Microsoft.Compute/virtualMachines/write",
                "Microsoft.Compute/virtualMachines/delete",
                "Microsoft.Network/register/action",
                "Microsoft.Network/locations/operations/read",
                "Microsoft.Network/locations/operationResults/read",
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
                "Microsoft.Resources/register/action",
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

export const PROVIDER_SETUP_INSTRUCTIONS: Record<string, ProviderSetupInstructions> = {
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
        text: "For a quick setup, attach the AmazonEC2FullAccess and AmazonSSMReadOnlyAccess managed policies. For fine-grained access, choose Attach policies directly → Create policy, paste the JSON policy below into the JSON editor, save it, and attach it to the user. The JSON policy also lets ByocVPN verify its permissions from Settings.",
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
        text: "Assign permissions to the service account. Quick setup: attach the built-in roles Compute Admin and Service Usage Admin. Least-privilege setup: go to IAM & Admin → Roles → Create role, add each permission listed in the YAML below, then assign that custom role to the service account.",
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
