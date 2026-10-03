import { useEffect, useRef, useState } from "react";
import { VpnStatus } from "../types";

export function useRecentlyDisconnectedInstance(vpnStatus: VpnStatus) {
  const [recentlyDisconnectedInstanceId, setRecentlyDisconnectedInstanceId] =
    useState<string | null>(null);
  const previousConnectedInstanceIdRef = useRef<string | null>(null);

  const connectedInstanceId = vpnStatus.connected
    ? (vpnStatus.instance?.instanceId ?? null)
    : null;

  useEffect(() => {
    const previousConnectedInstanceId = previousConnectedInstanceIdRef.current;
    previousConnectedInstanceIdRef.current = connectedInstanceId;

    if (
      previousConnectedInstanceId !== null &&
      previousConnectedInstanceId !== connectedInstanceId
    ) {
      setRecentlyDisconnectedInstanceId(previousConnectedInstanceId);
      return;
    }
    if (connectedInstanceId !== null) {
      setRecentlyDisconnectedInstanceId((current) =>
        current === connectedInstanceId ? null : current,
      );
    }
  }, [connectedInstanceId]);

  function clearRecentlyDisconnectedInstance() {
    setRecentlyDisconnectedInstanceId(null);
  }

  return { recentlyDisconnectedInstanceId, clearRecentlyDisconnectedInstance };
}
