import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink } from "lucide-react";
import { useUpdateNotice } from "../../hooks/useUpdateNotice";
import { Banner } from "../primitives/Banner";
import { Button } from "../primitives/Button";

export function UpdateBanner() {
  const { availableUpdate, dismissUpdate } = useUpdateNotice();

  if (availableUpdate === null) {
    return null;
  }

  function handleOpenRelease() {
    if (availableUpdate !== null) {
      void openUrl(availableUpdate.releaseUrl);
    }
  }

  return (
    <Banner
      variant="info"
      title={`Version ${availableUpdate.version} is available`}
      onDismiss={() => void dismissUpdate()}
      className="mb-3"
    >
      <div className="flex flex-col items-start gap-2">
        <p>
          A new release is available to download on GitHub. Updates are never installed
          automatically.
        </p>
        <Button
          variant="secondary"
          size="md"
          icon={<ExternalLink size={14} />}
          onClick={handleOpenRelease}
        >
          View release
        </Button>
      </div>
    </Banner>
  );
}
