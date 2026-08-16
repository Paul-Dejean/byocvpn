import { CloudProviderName } from "../../types";

export const PROVIDER_STRIPE: Record<CloudProviderName, string> = {
  [CloudProviderName.Aws]: "border-l-orange-500",
  [CloudProviderName.Oracle]: "border-l-red-500",
  [CloudProviderName.Gcp]: "border-l-blue-500",
  [CloudProviderName.Azure]: "border-l-sky-500",
};
