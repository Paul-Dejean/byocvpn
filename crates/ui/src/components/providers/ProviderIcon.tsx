interface ProviderIconProps {
  provider: string;
  className?: string;
}

const PROVIDER_ICONS: Record<string, { src: string; alt: string }> = {
  aws: { src: "/cloud-providers/aws-icon.svg", alt: "AWS" },
  gcp: { src: "/cloud-providers/google-cloud-icon.svg", alt: "GCP" },
  azure: { src: "/cloud-providers/azure-icon.svg", alt: "Azure" },
  oracle: { src: "/cloud-providers/oracle-icon.svg", alt: "Oracle" },
};

export function ProviderIcon({ provider, className = "w-8 h-8" }: ProviderIconProps) {
  const icon = PROVIDER_ICONS[provider.toLowerCase()];
  if (!icon) return null;
  return (
    <img src={icon.src} alt={icon.alt} className={`object-contain ${className}`} />
  );
}
