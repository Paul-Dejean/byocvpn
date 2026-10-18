import { RefreshCw } from "lucide-react";
import { useUpdateNotice } from "../../hooks/useUpdateNotice";
import { Toggle } from "../primitives/Toggle";
import { SettingsRow } from "./SettingsRow";

export function UpdateCheckCard() {
  const { isCheckEnabled, setCheckEnabled } = useUpdateNotice();

  return (
    <SettingsRow
      icon={<RefreshCw size={16} />}
      title="Check for updates"
      description="Look for a newer release on GitHub when the app starts"
      control={
        <Toggle
          checked={isCheckEnabled}
          onChange={() => void setCheckEnabled(!isCheckEnabled)}
          ariaLabel="Toggle update checks"
        />
      }
    >
      <p className="text-caption text-fg-moderate">
        Sends one request to GitHub's public API and shows a notice when a newer version exists.
        Nothing is downloaded or installed automatically.
      </p>
    </SettingsRow>
  );
}
