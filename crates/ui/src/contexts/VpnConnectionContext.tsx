import { createContext, useContext, ReactNode } from "react";
import { Instance } from "../types";
import { useVpnConnection } from "../hooks";
import { VpnStatus } from "../hooks/useVpnConnection";

interface VpnConnectionContextValue {
  vpnStatus: VpnStatus;
  checkVpnStatus: () => Promise<void>;
  isConnecting: boolean;
  error: string | null;
  connectToVpn: (instance: Instance) => Promise<void>;
  disconnectFromVpn: () => Promise<void>;
  clearError: () => void;
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

  return (
    <VpnConnectionContext.Provider value={vpnConnection}>
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
