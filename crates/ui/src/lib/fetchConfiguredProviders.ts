import { CloudProviderName } from "../types";
import { invokeCommand } from "./invokeCommand";

export async function fetchConfiguredProviders(): Promise<CloudProviderName[]> {
  const checks = await Promise.all(
    Object.values(CloudProviderName).map(async (provider) => {
      try {
        const credentials = await invokeCommand("get_credentials", {
          provider,
        });
        return credentials !== null ? provider : null;
      } catch {
        return null;
      }
    }),
  );
  return checks.filter(
    (provider): provider is CloudProviderName => provider !== null,
  );
}
