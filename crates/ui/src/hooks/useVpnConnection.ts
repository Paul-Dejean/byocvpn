import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import { Instance } from "../types";

export enum ServerStatus {
  IDLE = "idle",
  CONNECTING = "connecting",
  CONNECTED = "connected",
}

export interface VpnMetrics {
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  uploadRate: number;
  downloadRate: number;
}

export interface VpnStatus {
  connected: boolean;
  instance: Instance | null;
  metrics: VpnMetrics | null;
}

const initialVpnStatus: VpnStatus = {
  connected: false,
  instance: null,
  metrics: null,
};

export const useVpnConnection = () => {
  const [vpnStatus, setVpnStatus] = useState<VpnStatus>(initialVpnStatus);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkVpnStatus();

    const unlistenPromise = listen<VpnStatus>("vpn-status", (event) => {
      setVpnStatus(event.payload);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const checkVpnStatus = async () => {
    try {
      const status = await invoke<VpnStatus>("get_vpn_status");
      setVpnStatus(status);
      if (status.connected) {
        invoke("start_metrics_stream").catch((error) =>
          console.error("Failed to resume metrics stream:", error),
        );
      }
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
      toast.success("Connected to VPN successfully!");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to connect to VPN";
      setError(errorMessage);
      console.error("Failed to connect to VPN:", error);
      toast.error(errorMessage);
    } finally {
      setIsConnecting(false);
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
