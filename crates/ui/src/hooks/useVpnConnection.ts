import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import toast from "react-hot-toast";
import { Instance } from "../types";
import { useErrorLogContext } from "../contexts";

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
  const { addEntry } = useErrorLogContext();

  useEffect(() => {
    checkVpnStatus();

    const unlistenPromise = listen<VpnStatus>("vpn-status", (event) => {
      setVpnStatus(event.payload);
    });

    const unlistenFocusPromise = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) checkVpnStatus();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      unlistenFocusPromise.then((unlisten) => unlisten());
    };
  }, []);

  const checkVpnStatus = async () => {
    try {
      const status = await invoke<VpnStatus>("get_vpn_status");
      setVpnStatus(status);
      if (status.connected) {
        invoke("subscribe_to_vpn_status").catch((error) =>
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
        provider: selectedInstance.provider,
        publicIpV4: selectedInstance.publicIpV4 || null,
        publicIpV6: selectedInstance.publicIpV6 || null,
      });

      console.log("VPN connected:", response);
      toast.success("Connected to VPN successfully!");
    } catch (error) {
      const errorMessage = typeof error === "string" ? error : "Failed to connect to VPN";
      setError(errorMessage);
      console.error("Failed to connect to VPN:", error);
      toast.error(errorMessage);
      addEntry(errorMessage, "connect to VPN");
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
      const errorMessage = typeof error === "string" ? error : "Failed to disconnect from VPN";
      setError(errorMessage);
      toast.error(errorMessage);
      console.error("Failed to disconnect from VPN:", error);
      addEntry(errorMessage, "disconnect from VPN");
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
