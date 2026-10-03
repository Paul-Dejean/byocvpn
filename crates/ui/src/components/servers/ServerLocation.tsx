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
  layout?: "inline" | "stacked";
}

export function ServerLocation({
  provider,
  region,
  size = "md",
  layout = "inline",
}: ServerLocationProps) {
  const regionInfo = getRegionInfo(provider, region);
  const countryName = getCountryName(regionInfo.countryCode) || region;
  const textSize = size === "lg" ? "text-feature" : "text-body-sm";

  const providerTag = <Tag>{PROVIDER_METADATA[provider].shortLabel}</Tag>;

  return (
    <div
      className={`flex items-center gap-2 min-w-0 ${layout === "stacked" ? "flex-col" : ""}`}
    >
      <div className="flex items-center gap-2 min-w-0">
        <FlagIcon
          countryCode={regionInfo.countryCode}
          round
          size={size === "lg" ? 24 : 20}
        />
        <span className={`${textSize} text-fg-lighter truncate`}>{countryName}</span>
        <span className="text-fg-moderate">|</span>
        <span className={`${textSize} text-fg-moderate truncate`}>
          {regionInfo.city || region}
        </span>
        {layout === "inline" && providerTag}
      </div>
      {layout === "stacked" && providerTag}
    </div>
  );
}
