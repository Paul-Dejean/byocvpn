import { useState } from "react";
import { commands } from "../bindings";
import { CloudProviderName, Permissions } from "../types";

export function usePermissions() {
  const [permissions, setPermissions] = useState<Permissions | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function verifyPermissions(provider: CloudProviderName): Promise<Permissions | null> {
    setIsVerifying(true);
    setError(null);
    setPermissions(null);
    const result = await commands.verifyPermissions(provider);
    setIsVerifying(false);
    if (result.status === "error") {
      setError(result.error);
      return null;
    }
    setPermissions(result.data);
    return result.data;
  }

  function clearPermissions(): void {
    setPermissions(null);
    setError(null);
  }

  return {
    permissions,
    isVerifying,
    error,
    verifyPermissions,
    clearPermissions,
  };
}
