import { createContext, useContext, ReactNode } from "react";
import { Instance } from "../types";
import { useVpnConnection } from "../hooks/useVpnConnection";
import { useRecentlyDisconnectedInstance } from "../hooks/useRecentlyDisconnectedInstance";
import { VpnStatus } from "../types";

interface VpnConnectionContextValue {
  vpnStatus: VpnStatus;
  checkVpnStatus: () => Promise<void>;
  isConnecting: boolean;
  isDisconnecting: boolean;
  isDaemonRunning: boolean;
  error: string | null;
  connectToVpn: (instance: Instance) => Promise<void>;
  disconnectFromVpn: () => Promise<void>;
  clearError: () => void;
  recentlyDisconnectedInstanceId: string | null;
  clearRecentlyDisconnectedInstance: () => void;
}

const VpnConnectionContext = createContext<VpnConnectionContextValue | null>(
  null,
);

interface VpnConnectionProviderProps {
  children: ReactNode;
}

export function VpnConnectionProvider({
  children,
}: VpnConnectionProviderProps) {
  const vpnConnection = useVpnConnection();
  const recentlyDisconnectedInstance = useRecentlyDisconnectedInstance(
    vpnConnection.vpnStatus,
  );

  return (
    <VpnConnectionContext.Provider
      value={{ ...vpnConnection, ...recentlyDisconnectedInstance }}
    >
      {children}
    </VpnConnectionContext.Provider>
  );
}

export function useVpnConnectionContext() {
  const context = useContext(VpnConnectionContext);
  if (!context) {
    throw new Error(
      "useVpnConnectionContext must be used within VpnConnectionProvider",
    );
  }
  return context;
}
