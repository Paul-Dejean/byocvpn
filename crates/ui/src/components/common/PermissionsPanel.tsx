import { Check, X } from "lucide-react";
import { Permissions } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Tag } from "../primitives/Tag";
import { Banner } from "../primitives/Banner";

interface PermissionsPanelProps {
  permissions: Permissions | null;
  isVerifying: boolean;
  error: string | null;
}

export function PermissionsPanel({
  permissions,
  isVerifying,
  error,
}: PermissionsPanelProps) {
  if (isVerifying) {
    return (
      <div className="flex items-center gap-2 text-body-sm text-fg-medium">
        <Spinner color="border-bd-strong" />
        Verifying permissions
      </div>
    );
  }

  if (error) {
    return <Banner variant="danger">{error}</Banner>;
  }

  if (!permissions) {
    return null;
  }

  const missingCount = permissions.filter((status) => !status.granted).length;
  const allGranted = missingCount === 0;

  return (
    <div className="rounded-xl border border-bd-moderate bg-bg-bolder p-4 flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <h4 className="text-body-sm text-fg-lighter">Permissions</h4>
        {allGranted ? (
          <Tag tone="success">All {permissions.length} granted</Tag>
        ) : (
          <Tag tone="warning">{missingCount} missing</Tag>
        )}
      </div>
      <ul className="flex flex-col gap-1.5">
        {permissions.map((status) => (
          <li key={status.permission} className="flex items-center gap-2 text-caption">
            {status.granted ? (
              <Check size={14} className="text-fg-success-moderate flex-shrink-0" />
            ) : (
              <X size={14} className="text-fg-danger-moderate flex-shrink-0" />
            )}
            <span className={status.granted ? "text-fg-medium" : "text-fg-danger-moderate"}>
              {status.permission}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
