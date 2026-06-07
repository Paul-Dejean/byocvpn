import { CloudProviderName } from "../../types";

interface ProviderIconProps {
  provider: string;
  className?: string;
}

const PROVIDER_ICONS: Record<CloudProviderName, { src: string; alt: string }> = {
  [CloudProviderName.Aws]: { src: "/cloud-providers/aws-icon.svg", alt: "AWS" },
  [CloudProviderName.Gcp]: { src: "/cloud-providers/google-cloud-icon.svg", alt: "GCP" },
  [CloudProviderName.Azure]: { src: "/cloud-providers/azure-icon.svg", alt: "Azure" },
  [CloudProviderName.Oracle]: { src: "/cloud-providers/oracle-icon.svg", alt: "Oracle" },
};

export function ProviderIcon({ provider, className = "w-8 h-8" }: ProviderIconProps) {
  const icon = PROVIDER_ICONS[provider as CloudProviderName];
  if (!icon) return null;
  return (
    <img src={icon.src} alt={icon.alt} className={`object-contain ${className}`} />
  );
}
