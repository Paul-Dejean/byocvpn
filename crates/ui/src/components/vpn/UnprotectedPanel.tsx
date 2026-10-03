import { ShieldOff } from "lucide-react";
import { useNetworkLocation } from "../../hooks/useNetworkLocation";
import { FlagIcon } from "../FlagIcon";
import { Spinner } from "../primitives/Spinner";
import { IpAddressesCard } from "./IpAddressesCard";
import { StatusHero } from "./StatusHero";

interface UnprotectedPanelProps {
  hasServers: boolean;
}

export function UnprotectedPanel({ hasServers }: UnprotectedPanelProps) {
  const { location, isLoading } = useNetworkLocation(false);
  const hasLocation = location !== null && (location.country || location.city);

  return (
    <div className="h-full flex flex-col gap-3">
      <StatusHero
        tone="unprotected"
        icon={<ShieldOff size={48} strokeWidth={1.5} className="text-fg-danger-moderate" />}
        title="Unprotected"
      >
        <p className="text-body-sm text-fg-medium max-w-[260px]">
          {hasServers
            ? "Anyone can see what you browse. Connect to a server to go private."
            : "Your connection is exposed. Add a server to stay private."}
        </p>
      </StatusHero>

      <section className="rounded-lg bg-bg-medium p-4 flex flex-col gap-1">
        <span className="text-caption text-fg-medium">Current location</span>
        {isLoading ? (
          <Spinner size="w-4 h-4" color="border-bd-strong" />
        ) : hasLocation && location ? (
          <span className="flex items-center gap-2 text-body-sm text-fg-lighter">
            <FlagIcon countryCode={location.countryCode} />
            {location.country}
            {location.city ? `, ${location.city}` : ""}
          </span>
        ) : (
          <span className="text-body-sm text-fg-moderate">Unavailable</span>
        )}
      </section>

      <IpAddressesCard
        ipV4={location?.publicIpV4 ?? null}
        ipV6={location?.publicIpV6 ?? null}
      />
    </div>
  );
}
