import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { Instance } from "../types";

/**
 * VPN server status states
 */
export enum ServerStatus {
  IDLE = "idle",
  CONNECTING = "connecting",
  CONNECTED = "connected",
}

/**
 * VPN status response from backend
 */

type VpnStatus =
  | {
      connected: false;
    }
  | {
      connected: true;
      instance: Instance;
    };

export const useVpnConnection = () => {
  const [vpnStatus, setVpnStatus] = useState<VpnStatus>({
    connected: false,
  });
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkVpnStatus = async () => {
    try {
      const status = await invoke<VpnStatus>("get_vpn_status");
      console.log("VPN Status on mount:", status);
      setVpnStatus(status);
    } catch (error) {
      console.error("Failed to check VPN status:", error);
    }
  };

  const connectToVpn = async (selectedInstance: Instance) => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setError(null);

    try {
      const response = await invoke("connect", {
        instanceId: selectedInstance.id,
        region: selectedInstance.region,
      });

      console.log("VPN connected:", response);
      console.log("Setting serverStatus to 'connected'");
      toast.success("Connected to VPN successfully!");
      await checkVpnStatus();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to connect to VPN";
      setError(errorMessage);
      console.error("Failed to connect to VPN:", error);
      toast.error(errorMessage);
    }
  };

  const disconnectFromVpn = async () => {
    setError(null);

    try {
      const response = await invoke("disconnect");
      console.log("VPN disconnected:", response);

      toast.success("Disconnected from VPN");
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Failed to disconnect from VPN";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to disconnect from VPN:", error);
    }
  };

  const clearError = () => {
    setError(null);
  };

  return {
    vpnStatus,
    checkVpnStatus,
    isConnecting,
    error,
    connectToVpn,
    disconnectFromVpn,
    clearError,
  };
};
