import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { CloudProviderName, Permissions } from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { Banner } from "../primitives/Banner";
import { Button } from "../primitives/Button";
import { Drawer } from "../primitives/Drawer";
import { PermissionsPanel } from "./PermissionsPanel";

interface PermissionsDrawerProps {
  provider: CloudProviderName | null;
  permissions: Permissions | null;
  isVerifying: boolean;
  error: string | null;
  onClose: () => void;
  onRetry: () => void;
}

const SIMULATE_POLICY_ACTION = "iam:SimulatePrincipalPolicy";

const SIMULATE_POLICY_STATEMENT = JSON.stringify(
  {
    Effect: "Allow",
    Action: SIMULATE_POLICY_ACTION,
    Resource: "arn:aws:iam::*:user/${aws:username}",
  },
  null,
  2,
);

export function PermissionsDrawer({
  provider,
  permissions,
  isVerifying,
  error,
  onClose,
  onRetry,
}: PermissionsDrawerProps) {
  const providerLabel = provider ? PROVIDER_METADATA[provider].shortLabel : "";
  const isMissingSimulatePermission =
    provider === CloudProviderName.Aws && error !== null && error.includes(SIMULATE_POLICY_ACTION);

  return (
    <Drawer
      isOpen={provider !== null}
      onClose={onClose}
      title={`${providerLabel} permissions`}
      subtitle="Checks whether your credentials can do everything ByocVPN needs"
      footer={
        <div className="flex gap-3">
          <Button variant="secondary" size="lg" onClick={onClose} className="flex-1">
            Close
          </Button>
          <Button
            variant="primary"
            size="lg"
            onClick={onRetry}
            loading={isVerifying}
            className="flex-1"
          >
            Verify again
          </Button>
        </div>
      }
    >
      {isMissingSimulatePermission ? (
        <MissingSimulatePermission />
      ) : (
        <PermissionsPanel permissions={permissions} isVerifying={isVerifying} error={error} />
      )}
    </Drawer>
  );
}

function MissingSimulatePermission() {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(SIMULATE_POLICY_STATEMENT);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="flex flex-col gap-4">
      <Banner variant="info" title="Verification needs one extra permission">
        Your access key cannot call {SIMULATE_POLICY_ACTION}, which ByocVPN uses to ask
        AWS what the key is allowed to do. This is optional: deployments work without it,
        and any missing permission will surface as an error at that point.
      </Banner>
      <div className="flex flex-col gap-2">
        <p className="text-caption text-fg-medium">
          To enable verification, add this read-only statement to the policy attached to
          your IAM user. It only lets the key inspect its own permissions.
        </p>
        <div className="rounded-lg bg-bg-medium border border-bd-moderate overflow-hidden">
          <div className="flex items-center justify-end px-3 py-2 border-b border-bd-faint">
            <Button
              variant="secondary"
              size="md"
              onClick={handleCopy}
              icon={
                copied ? (
                  <Check size={14} className="text-fg-success-moderate" />
                ) : (
                  <Copy size={14} />
                )
              }
            >
              {copied ? "Copied" : "Copy"}
            </Button>
          </div>
          <pre className="p-3 text-caption font-mono text-fg-medium overflow-auto">
            {SIMULATE_POLICY_STATEMENT}
          </pre>
        </div>
      </div>
    </div>
  );
}
