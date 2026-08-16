import { useCallback, useState } from "react";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { invokeCommand } from "../lib/invokeCommand";
import { Permissions, CloudProviderName } from "../types";
import {
  AwsCredentials,
  GcpCredentials,
  AzureCredentials,
} from "./useCredentials";

type VerifiableCredentials = AwsCredentials | GcpCredentials | AzureCredentials;

export function usePermissions() {
  const [permissions, setPermissions] = useState<Permissions | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const verifyPermissions = useCallback(
    async (
      provider: CloudProviderName,
      credentials?: VerifiableCredentials,
    ): Promise<Permissions | null> => {
      setIsVerifying(true);
      setError(null);
      try {
        const result = await invokeCommand<Permissions>("verify_permissions", {
          provider,
          credentials: credentials ? { provider, ...credentials } : null,
        });
        setPermissions(result);
        return result;
      } catch (error) {
        setError(extractErrorMessage(error, "Failed to verify permissions"));
        return null;
      } finally {
        setIsVerifying(false);
      }
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
