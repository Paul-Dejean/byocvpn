import { CloudProviderName } from "../types";

interface ProviderMetadata {
  label: string;
  shortLabel: string;
  description: string;
}

export const PROVIDER_METADATA: Record<CloudProviderName, ProviderMetadata> = {
  [CloudProviderName.Aws]: {
    label: "Amazon Web Services",
    shortLabel: "AWS",
    description: "Deploy on EC2 — available in 30+ regions worldwide",
  },
  [CloudProviderName.Azure]: {
    label: "Microsoft Azure",
    shortLabel: "Azure",
    description: "Deploy on Azure Virtual Machines — available in 60+ regions worldwide",
  },
  [CloudProviderName.Gcp]: {
    label: "Google Cloud Platform",
    shortLabel: "GCP",
    description: "Deploy on Compute Engine using a service account — available in 40+ regions worldwide",
  },
  [CloudProviderName.Oracle]: {
    label: "Oracle Cloud Infrastructure",
    shortLabel: "Oracle",
    description: "Deploy on OCI Compute — includes an Always Free tier",
  },
};
