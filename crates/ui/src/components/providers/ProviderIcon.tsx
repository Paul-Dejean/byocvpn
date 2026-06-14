import { CloudProviderName } from "../../types";

interface ProviderIconProps {
  provider: CloudProviderName;
  className?: string;
}

const PROVIDER_ICONS: Partial<
  Record<CloudProviderName, { src: string; alt: string }>
> = {
  [CloudProviderName.Gcp]: {
    src: "/cloud-providers/google-cloud-icon.svg",
    alt: "GCP",
  },
  [CloudProviderName.Azure]: {
    src: "/cloud-providers/azure-icon.svg",
    alt: "Azure",
  },
  [CloudProviderName.Oracle]: {
    src: "/cloud-providers/oracle-icon.svg",
    alt: "Oracle",
  },
};

export function ProviderIcon({
  provider,
  className = "w-8 h-8",
}: ProviderIconProps) {
  if (provider === CloudProviderName.Aws) {
    return (
      <>
        <img
          src="/cloud-providers/aws-icon-dark.svg"
          alt="AWS"
          className={`object-contain light:hidden ${className}`}
        />
        <img
          src="/cloud-providers/aws-icon-light.svg"
          alt="AWS"
          className={`object-contain hidden light:block ${className}`}
        />
      </>
    );
  }

  const icon = PROVIDER_ICONS[provider];
  if (!icon) {
    return null;
  }

  return (
    <img
      src={icon.src}
      alt={icon.alt}
      className={`object-contain ${className}`}
    />
  );
}
