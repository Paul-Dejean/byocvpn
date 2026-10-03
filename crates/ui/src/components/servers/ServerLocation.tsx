import { CloudProviderName } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { PROVIDER_METADATA } from "../../constants/providers";
import { getCountryName } from "../../lib/countryName";
import { FlagIcon } from "../FlagIcon";
import { Tag } from "../primitives/Tag";

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
  const textSize = size === "lg" ? "text-body" : "text-body-sm";

  return (
    <div className="flex items-center gap-2 min-w-0">
      <FlagIcon countryCode={regionInfo.countryCode} round />
      <span className={`${textSize} text-fg-lighter truncate`}>{countryName}</span>
      <span className="text-fg-moderate">|</span>
      <span className={`${textSize} text-fg-medium truncate`}>
        {regionInfo.city || region}
      </span>
      <Tag>{PROVIDER_METADATA[provider].shortLabel}</Tag>
    </div>
  );
}
