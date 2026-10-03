import { Pencil, Plus, RefreshCw, Trash2, Zap } from "lucide-react";
import { CloudProviderName } from "../../types";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";

interface AccountRowProps {
  provider: CloudProviderName;
  title: string;
  hasCredentials: boolean | null;
  isProvisioned: boolean;
  isConfirmingDelete: boolean;
  onEdit: () => void;
  onProvision: () => void;
  onRequestDelete: () => void;
  onCancelDelete: () => void;
  onConfirmDelete: () => void;
}

export function AccountRow({
  provider,
  title,
  hasCredentials,
  isProvisioned,
  isConfirmingDelete,
  onEdit,
  onProvision,
  onRequestDelete,
  onCancelDelete,
  onConfirmDelete,
}: AccountRowProps) {
  return (
    <div className="p-4 flex items-center justify-between gap-4">
      <div className="flex items-center gap-3 min-w-0">
        <AccountIconTile provider={provider} />
        <div className="flex items-center gap-2 min-w-0">
          <h3 className="text-sm text-primary truncate">{title}</h3>
          {hasCredentials && (
            <StatusPill isProvisioned={isProvisioned} />
          )}
        </div>
      </div>

      {hasCredentials === null ? (
        <Spinner color="border-gray-400" />
      ) : hasCredentials ? (
        <div className="flex items-center gap-2">
          {isConfirmingDelete ? (
            <>
              <span className="text-xs text-gray-300">Delete credentials?</span>
              <Button
                variant="secondary"
                size="none"
                onClick={onCancelDelete}
                className="px-3 py-1.5 text-sm"
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="none"
                onClick={onConfirmDelete}
                className="px-3 py-1.5 text-sm"
              >
                Confirm
              </Button>
            </>
          ) : (
            <>
              <IconButton
                accent={isProvisioned ? "blue" : "amber"}
                size="sm"
                onClick={onProvision}
                title={isProvisioned ? "Re-provision" : "Provision"}
              >
                {isProvisioned ? <RefreshCw size={16} /> : <Zap size={16} />}
              </IconButton>
              <IconButton
                accent="red"
                size="sm"
                onClick={onRequestDelete}
                title="Delete credentials"
              >
                <Trash2 size={16} />
              </IconButton>
              <Button
                variant="secondary"
                size="none"
                onClick={onEdit}
                icon={<Pencil size={14} />}
                className="px-3 py-1.5 text-sm"
              >
                Edit
              </Button>
            </>
          )}
        </div>
      ) : (
        <Button
          variant="primary"
          size="none"
          onClick={onEdit}
          icon={<Plus size={14} />}
          className="px-3 py-1.5 text-sm"
        >
          Add account
        </Button>
      )}
    </div>
  );
}

export function AccountIconTile({ provider }: { provider: CloudProviderName }) {
  return (
    <div className="w-9 h-9 rounded-lg bg-gray-700 border border-gray-500/60 flex items-center justify-center flex-shrink-0 p-2">
      <ProviderIcon provider={provider} className="w-full h-full" />
    </div>
  );
}

function StatusPill({ isProvisioned }: { isProvisioned: boolean }) {
  return isProvisioned ? (
    <span className="px-2 py-0.5 rounded-full bg-success-900/50 text-[11px] text-success-300 flex-shrink-0">
      Provisioned
    </span>
  ) : (
    <span className="px-2 py-0.5 rounded-full bg-warning-900/50 text-[11px] text-warning-300 flex-shrink-0">
      Not provisioned
    </span>
  );
}
