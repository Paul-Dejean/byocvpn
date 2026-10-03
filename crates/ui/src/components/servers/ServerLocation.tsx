import { CloudProviderName } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { PROVIDER_METADATA } from "../../constants/providers";
import { getCountryName } from "../../lib/countryName";
import { FlagIcon } from "../FlagIcon";

interface ServerLocationProps {
  provider: CloudProviderName;
  region: string;
  size?: "md" | "lg";
}

export function ServerLocation({
  provider,
  region,
  size = "md",
}: ServerLocationProps) {
  const regionInfo = getRegionInfo(provider, region);
  const countryName = getCountryName(regionInfo.countryCode) || region;
  const textSize = size === "lg" ? "text-base" : "text-sm";

  return (
    <div className="flex items-center gap-2 min-w-0">
      <FlagIcon countryCode={regionInfo.countryCode} round />
      <span className={`${textSize} text-primary truncate`}>{countryName}</span>
      <span className="text-gray-500">|</span>
      <span className={`${textSize} text-gray-300 truncate`}>
        {regionInfo.city || region}
      </span>
      <span className="ml-1 px-2 py-0.5 rounded-full bg-gray-600 text-[11px] text-gray-100 flex-shrink-0">
        {PROVIDER_METADATA[provider].shortLabel}
      </span>
    </div>
  );
}
