import { Check, X } from "lucide-react";
import { Permissions } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Alert } from "../primitives/Alert";

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
      <div className="flex items-center gap-2 text-sm text-gray-300">
        <Spinner color="border-gray-400" />
        Verifying permissions
      </div>
    );
  }

  if (error) {
    return <Alert variant="error">{error}</Alert>;
  }

  if (!permissions) {
    return null;
  }

  const missingCount = permissions.filter((status) => !status.granted).length;
  const allGranted = missingCount === 0;

  return (
    <div className="rounded-xl border border-gray-500/50 bg-gray-750 p-4 flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <h4 className="text-sm text-primary">Permissions</h4>
        {allGranted ? (
          <span className="px-2 py-0.5 rounded-full bg-success-900/50 text-[11px] text-success-300">
            All {permissions.length} granted
          </span>
        ) : (
          <span className="px-2 py-0.5 rounded-full bg-warning-900/50 text-[11px] text-warning-300">
            {missingCount} missing
          </span>
        )}
      </div>
      <ul className="flex flex-col gap-1.5">
        {permissions.map((status) => (
          <li key={status.permission} className="flex items-center gap-2 text-xs">
            {status.granted ? (
              <Check size={14} className="text-success-400 flex-shrink-0" />
            ) : (
              <X size={14} className="text-danger-400 flex-shrink-0" />
            )}
            <span className={status.granted ? "text-gray-200" : "text-danger-300"}>
              {status.permission}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
