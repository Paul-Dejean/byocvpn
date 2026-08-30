import type { CloudProviderName as CloudProviderNameBinding } from "../bindings";

export const CloudProviderName = {
  Aws: "AWS",
  Oracle: "ORACLE",
  Gcp: "GCP",
  Azure: "AZURE",
} as const satisfies Record<string, CloudProviderNameBinding>;

export type CloudProviderName = CloudProviderNameBinding;
