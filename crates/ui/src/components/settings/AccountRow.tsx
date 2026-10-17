import { ListChecks, Pencil, Plus, Trash2 } from "lucide-react";
import { CloudProviderName } from "../../types";
import { ProviderIcon } from "../providers/ProviderIcon";
import { Spinner } from "../primitives/Spinner";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";

interface AccountRowProps {
  provider: CloudProviderName;
  title: string;
  hasCredentials: boolean | null;
  isConfirmingDelete: boolean;
  onEdit: () => void;
  onVerify?: () => void;
  onRequestDelete: () => void;
  onCancelDelete: () => void;
  onConfirmDelete: () => void;
}

export function AccountRow({
  provider,
  title,
  hasCredentials,
  isConfirmingDelete,
  onEdit,
  onVerify,
  onRequestDelete,
  onCancelDelete,
  onConfirmDelete,
}: AccountRowProps) {
  return (
    <div className="flex items-center justify-between gap-4 p-4">
      <div className="flex min-w-0 items-center gap-3">
        <AccountIconTile provider={provider} />
        <h3 className="truncate text-body-sm text-fg-lighter">{title}</h3>
      </div>

      {hasCredentials === null ? (
        <Spinner color="border-bd-strong" />
      ) : hasCredentials ? (
        <div className="flex items-center gap-2">
          {isConfirmingDelete ? (
            <>
              <span className="text-caption text-fg-medium">Delete credentials?</span>
              <Button variant="secondary" size="lg" onClick={onCancelDelete}>
                Cancel
              </Button>
              <Button variant="danger" size="lg" onClick={onConfirmDelete}>
                Confirm
              </Button>
            </>
          ) : (
            <>
              {onVerify && (
                <IconButton accent="brand" size="sm" onClick={onVerify} title="Verify permissions">
                  <ListChecks size={16} />
                </IconButton>
              )}
              <IconButton
                accent="danger"
                size="sm"
                onClick={onRequestDelete}
                title="Delete credentials"
              >
                <Trash2 size={16} />
              </IconButton>
              <Button variant="secondary" size="lg" onClick={onEdit} icon={<Pencil size={14} />}>
                Edit
              </Button>
            </>
          )}
        </div>
      ) : (
        <Button variant="primary" size="lg" onClick={onEdit} icon={<Plus size={14} />}>
          Add account
        </Button>
      )}
    </div>
  );
}

export function AccountIconTile({ provider }: { provider: CloudProviderName }) {
  return (
    <div className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-bd-moderate bg-bg-medium p-2">
      <ProviderIcon provider={provider} className="size-full" />
    </div>
  );
}
