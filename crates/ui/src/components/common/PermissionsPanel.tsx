import { Permissions } from "../../types";
import { Spinner } from "../primitives/Spinner";
import { Badge } from "../primitives/Badge";
import { Alert } from "../primitives/Alert";

interface PermissionsPanelProps {
  permissions: Permissions | null;
  isVerifying: boolean;
  error: string | null;
}

function CheckIcon() {
  return (
    <svg
      className="w-3.5 h-3.5 text-success-300 flex-shrink-0"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2.5}
        d="M5 13l4 4L19 7"
      />
    </svg>
  );
}

function CrossIcon() {
  return (
    <svg
      className="w-3.5 h-3.5 text-danger-300 flex-shrink-0"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2.5}
        d="M6 18L18 6M6 6l12 12"
      />
    </svg>
  );
}

export function PermissionsPanel({
  permissions,
  isVerifying,
  error,
}: PermissionsPanelProps) {
  if (isVerifying) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-400 mt-3">
        <Spinner color="border-gray-400" />
        Verifying permissions…
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="error" className="mt-3">
        {error}
      </Alert>
    );
  }

  if (!permissions) {
    return null;
  }

  const missingCount = permissions.filter((status) => !status.granted).length;
  const allGranted = missingCount === 0;

  return (
    <div className="mt-3 rounded-xl border border-gray-500/15 bg-gray-800/40 p-4">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-semibold text-primary">Permissions</h4>
        {allGranted ? (
          <Badge variant="success" shape="pill">
            All {permissions.length} granted
          </Badge>
        ) : (
          <Badge variant="warning" shape="pill">
            {missingCount} missing
          </Badge>
        )}
      </div>
      <ul className="mt-3 flex flex-col gap-y-1.5">
        {permissions.map((status) => (
          <li
            key={status.permission}
            className="flex items-center gap-2 text-xs"
          >
            {status.granted ? <CheckIcon /> : <CrossIcon />}
            <span
              className={
                status.granted ? "text-gray-300" : "text-danger-300 font-medium"
              }
            >
              {status.permission}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
