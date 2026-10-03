import { Pencil, Plus, RefreshCw, Trash2, Zap } from "lucide-react";
import { CloudProviderName } from "../../types";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { Tag } from "../primitives/Tag";

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
          <h3 className="text-body-sm text-fg-lighter truncate">{title}</h3>
          {hasCredentials && (
            <StatusPill isProvisioned={isProvisioned} />
          )}
        </div>
      </div>

      {hasCredentials === null ? (
        <Spinner color="border-bd-strong" />
      ) : hasCredentials ? (
        <div className="flex items-center gap-2">
          {isConfirmingDelete ? (
            <>
              <span className="text-caption text-fg-medium">Delete credentials?</span>
              <Button
                variant="secondary"
                size="lg"
                onClick={onCancelDelete}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                size="lg"
                onClick={onConfirmDelete}
              >
                Confirm
              </Button>
            </>
          ) : (
            <>
              <IconButton
                accent={isProvisioned ? "brand" : "warning"}
                size="sm"
                onClick={onProvision}
                title={isProvisioned ? "Re-provision" : "Provision"}
              >
                {isProvisioned ? <RefreshCw size={16} /> : <Zap size={16} />}
              </IconButton>
              <IconButton
                accent="danger"
                size="sm"
                onClick={onRequestDelete}
                title="Delete credentials"
              >
                <Trash2 size={16} />
              </IconButton>
              <Button
                variant="secondary"
                size="lg"
                onClick={onEdit}
                icon={<Pencil size={14} />}
              >
                Edit
              </Button>
            </>
          )}
        </div>
      ) : (
        <Button
          variant="primary"
          size="lg"
          onClick={onEdit}
          icon={<Plus size={14} />}
        >
          Add account
        </Button>
      )}
    </div>
  );
}

export function AccountIconTile({ provider }: { provider: CloudProviderName }) {
  return (
    <div className="w-9 h-9 rounded-lg bg-bg-medium border border-bd-moderate flex items-center justify-center flex-shrink-0 p-2">
      <ProviderIcon provider={provider} className="w-full h-full" />
    </div>
  );
}

function StatusPill({ isProvisioned }: { isProvisioned: boolean }) {
  return isProvisioned ? (
    <Tag tone="success">Provisioned</Tag>
  ) : (
    <Tag tone="warning">Not provisioned</Tag>
  );
}
