import { CloudProviderName } from "../../../types";

export const PROVIDER_TILE_LABELS: Record<CloudProviderName, string> = {
  [CloudProviderName.Aws]: "AWS Account",
  [CloudProviderName.Oracle]: "Oracle Cloud",
  [CloudProviderName.Gcp]: "Google Cloud",
  [CloudProviderName.Azure]: "Microsoft Azure",
};
