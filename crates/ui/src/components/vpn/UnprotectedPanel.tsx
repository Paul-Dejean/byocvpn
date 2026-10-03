import { ShieldOff } from "lucide-react";
import { useNetworkLocation } from "../../hooks/useNetworkLocation";
import { FlagIcon } from "../FlagIcon";
import { Spinner } from "../primitives/Spinner";
import { IpAddressesCard } from "./IpAddressesCard";

interface UnprotectedPanelProps {
  hasServers: boolean;
}

export function UnprotectedPanel({ hasServers }: UnprotectedPanelProps) {
  const { location, isLoading } = useNetworkLocation(false);
  const hasLocation = location !== null && (location.country || location.city);

  return (
    <div className="h-full rounded-xl bg-gray-750 border border-gray-500/40 p-4 flex flex-col items-center">
      <div className="mt-6 w-24 h-24 rounded-full bg-danger-900/40 flex items-center justify-center">
        <ShieldOff size={40} strokeWidth={1.5} className="text-danger-400" />
      </div>

      <h2 className="mt-6 text-xl font-medium text-primary">Unprotected</h2>
      <p className="mt-2 text-sm text-gray-300 text-center max-w-[240px]">
        {hasServers
          ? "Anyone can see what you browse. Connect to a server to go private."
          : "Your connection is exposed. Add a server to stay private."}
      </p>

      <section className="mt-8 w-full rounded-lg bg-gray-700 p-3 flex flex-col gap-1">
        <span className="text-xs text-gray-300">Current location</span>
        {isLoading ? (
          <Spinner size="w-4 h-4" color="border-gray-400" />
        ) : hasLocation && location ? (
          <span className="flex items-center gap-2 text-sm text-primary">
            <FlagIcon countryCode={location.countryCode} />
            {location.country}
            {location.city ? `, ${location.city}` : ""}
          </span>
        ) : (
          <span className="text-sm text-gray-400">Unavailable</span>
        )}
      </section>

      <div className="mt-3 w-full">
        <IpAddressesCard
          ipV4={location?.publicIpV4 ?? null}
          ipV6={location?.publicIpV6 ?? null}
        />
      </div>
    </div>
  );
}
