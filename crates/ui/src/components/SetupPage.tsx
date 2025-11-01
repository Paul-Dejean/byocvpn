import { useState } from "react";
import { Page } from "../App";
import { invoke } from "@tauri-apps/api/core";

type CloudProvider = "aws" | "gcp" | "azure" | null;

interface CredentialsForm {
  accessKeyId: string;
  secretAccessKey: string;
}

export function SetupPage({ setPage }: { setPage: (page: Page) => void }) {
  // Currently, only AWS is available, GCP and Azure coming soon
  const [selectedProvider, setSelectedProvider] = useState<CloudProvider>(null);
  const [showPolicy, setShowPolicy] = useState(false);
  const [credentials, setCredentials] = useState<CredentialsForm>({
    accessKeyId: "",
    secretAccessKey: "",
  });
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleProviderSelect = (provider: CloudProvider) => {
    // Only allow AWS selection for now
    if (provider === "aws") {
      setSelectedProvider(provider);
      setShowPolicy(true);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setCredentials((prev) => ({
      ...prev,
      [name]: value,
    }));
  };

  const handleSetupClick = async () => {
    if (
      selectedProvider &&
      credentials.accessKeyId &&
      credentials.secretAccessKey
    ) {
      setIsSubmitting(true);
      console.log("Starting setup with:", { selectedProvider, credentials });
      try {
        // Save credentials to ~/.byocvpn/credentials
        const result = await invoke("save_credentials", {
          cloudProviderName: selectedProvider,
          accessKeyId: credentials.accessKeyId,
          secretAccessKey: credentials.secretAccessKey,
        });

        console.log("Credentials saved:", result);

        // Navigate to VPN page after successful save
        setPage(Page.VPN);
      } catch (error) {
        console.error("Failed to save credentials:", error);
        // You could show an error message to the user here
        setIsSubmitting(false);
      }
    } else {
      console.log("Please select a cloud provider and provide credentials");
    }
  };

  // AWS IAM Policy that needs to be applied
  const awsIamPolicy = `
 {
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "VPNPermissions",
      "Effect": "Allow",
      "Action": [
        "ec2:DescribeVpcs",
        "ec2:CreateVpc",
        "ec2:CreateTags",
        "ec2:CreateInternetGateway",
        "ec2:DescribeAvailabilityZones",
        "ec2:DescribeSubnets",
        "ec2:CreateSubnet",
        "ec2:ModifySubnetAttribute",
        "ec2:CreateSecurityGroup",
        "ec2:AuthorizeSecurityGroupIngress",
        "ec2:DescribeSecurityGroups",
        "ec2:RunInstances",
        "ec2:DescribeInstances",
        "ec2:TerminateInstances",
        "ssm:GetParameter"
      ],
      "Resource": "*"
    }
  ]
}`;

  return (
    <div className="flex flex-col items-center justify-center h-full bg-gray-900 text-white py-8">
      <div className="mb-8 w-full max-w-4xl">
        <h2 className="text-2xl font-semibold mb-4 text-center text-blue-400">
          Select a Cloud Provider
        </h2>
        <div className="flex flex-wrap justify-center gap-6">
          {/* AWS Card */}
          <div
            className={`w-52 p-6 border rounded-lg shadow-md cursor-pointer transition-all ${
              selectedProvider === "aws"
                ? "bg-gray-700 border-blue-500"
                : "bg-gray-800 hover:bg-gray-700 border-gray-700"
            }`}
            onClick={() => handleProviderSelect("aws")}
          >
            <div className="flex flex-col items-center">
              <div className="w-16 h-16 flex items-center justify-center mb-4 text-yellow-500 font-bold text-4xl">
                AWS
              </div>
              <h3 className="text-lg font-medium">Amazon Web Services</h3>
            </div>
          </div>

          {/* GCP Card */}
          <div
            className={`w-52 p-6 border rounded-lg shadow-md relative opacity-75 bg-gray-800 border-gray-700`}
          >
            {/* Coming Soon Tag */}
            <div className="absolute -top-2 -right-2 bg-amber-600 text-xs text-white font-bold px-2 py-1 rounded-md shadow-md">
              COMING SOON
            </div>
            <div className="flex flex-col items-center">
              <div className="w-16 h-16 flex items-center justify-center mb-4 text-red-400 font-bold text-4xl">
                GCP
              </div>
              <h3 className="text-lg font-medium">Google Cloud Platform</h3>
            </div>
          </div>

          {/* Azure Card */}
          <div
            className={`w-52 p-6 border rounded-lg shadow-md relative opacity-75 bg-gray-800 border-gray-700`}
          >
            {/* Coming Soon Tag */}
            <div className="absolute -top-2 -right-2 bg-amber-600 text-xs text-white font-bold px-2 py-1 rounded-md shadow-md">
              COMING SOON
            </div>
            <div className="flex flex-col items-center">
              <div className="w-16 h-16 flex items-center justify-center mb-4 text-blue-400 font-bold text-4xl">
                Azure
              </div>
              <h3 className="text-lg font-medium">Microsoft Azure</h3>
            </div>
          </div>
        </div>
      </div>

      {showPolicy && selectedProvider === "aws" && (
        <div className="w-full max-w-4xl px-6 py-5 mb-8 bg-gray-800 rounded-lg shadow-lg">
          <h3 className="text-xl font-semibold mb-3 text-blue-400">
            Step 1: Create IAM Policy
          </h3>
          <p className="text-gray-300 mb-4">
            Create an IAM policy in your AWS account with the following
            permissions. This policy allows the application to create and manage
            necessary AWS resources.
          </p>
          <div className="relative">
            <pre className="bg-gray-950 p-4 rounded-md text-xs text-gray-300 overflow-auto max-h-60">
              {awsIamPolicy}
            </pre>
            <button
              className="absolute top-2 right-2 px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700"
              onClick={() => {
                navigator.clipboard.writeText(awsIamPolicy);
              }}
            >
              Copy
            </button>
          </div>

          <h3 className="text-xl font-semibold mt-6 mb-3 text-blue-400">
            Step 2: Enter Credentials
          </h3>
          <p className="text-gray-300 mb-4">
            Create an IAM user with the above policy attached, then generate
            access keys and enter them below:
          </p>

          <div className="space-y-4">
            <div>
              <label
                htmlFor="accessKeyId"
                className="block text-sm font-medium text-gray-300 mb-1"
              >
                Access Key ID
              </label>
              <input
                type="text"
                id="accessKeyId"
                name="accessKeyId"
                value={credentials.accessKeyId}
                onChange={handleInputChange}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="AKIAIOSFODNN7EXAMPLE"
              />
            </div>

            <div>
              <label
                htmlFor="secretAccessKey"
                className="block text-sm font-medium text-gray-300 mb-1"
              >
                Secret Access Key
              </label>
              <input
                type="password"
                id="secretAccessKey"
                name="secretAccessKey"
                value={credentials.secretAccessKey}
                onChange={handleInputChange}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
              />
            </div>
          </div>
        </div>
      )}

      <button
        className={`px-6 py-2 text-white rounded transition ${
          selectedProvider &&
          credentials.accessKeyId &&
          credentials.secretAccessKey
            ? "bg-blue-600 hover:bg-blue-500 cursor-pointer"
            : "bg-gray-700 text-gray-400 cursor-not-allowed"
        }`}
        onClick={handleSetupClick}
        disabled={
          !selectedProvider ||
          !credentials.accessKeyId ||
          !credentials.secretAccessKey ||
          isSubmitting
        }
      >
        {isSubmitting ? "Setting up..." : "Start Setup"}
      </button>
    </div>
  );
}
// This component is designed to guide users through the initial setup of their Rust environment.
