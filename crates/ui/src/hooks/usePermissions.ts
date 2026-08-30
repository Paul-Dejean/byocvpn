import { useCallback, useState } from "react";
import { ProviderCredentials, commands } from "../bindings";
import {
  CloudProviderName,
  Permissions,
  VerifiableCredentials,
  VerifiableCredentialsMap,
  VerifiableProvider,
} from "../types";

type VerifyPermissionsFunction = {
  (provider: CloudProviderName): Promise<Permissions | null>;
  <Provider extends VerifiableProvider>(
    provider: Provider,
    credentials: VerifiableCredentialsMap[Provider],
  ): Promise<Permissions | null>;
};

export function usePermissions() {
  const [permissions, setPermissions] = useState<Permissions | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const verifyPermissions = useCallback<VerifyPermissionsFunction>(
    async (
      provider: CloudProviderName,
      credentials?: VerifiableCredentials,
    ): Promise<Permissions | null> => {
      setIsVerifying(true);
      setError(null);
      const result = await commands.verifyPermissions(
        provider,
        credentials
          ? ({ provider, ...credentials } as ProviderCredentials)
          : null,
      );
      setIsVerifying(false);
      if (result.status === "error") {
        setError(result.error);
        return null;
      }
      setPermissions(result.data);
      return result.data;
    },
    [],
  );

  const clearPermissions = useCallback((): void => {
    setPermissions(null);
    setError(null);
  }, []);

  return {
    permissions,
    isVerifying,
    error,
    verifyPermissions,
    clearPermissions,
  };
}
