import { useState, useEffect } from "react";
import { invokeCommand } from "../lib/invokeCommand";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
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

export type VpnStatus =
  | { connected: true; instance: Instance; metrics: VpnMetrics | null; connectionError: string | null }
  | { connected: false; instance: null; metrics: null; connectionError: string | null };

const initialVpnStatus: VpnStatus = {
  connected: false,
  instance: null,
  metrics: null,
  connectionError: null,
};

export function useVpnConnection() {
  const [vpnStatus, setVpnStatus] = useState<VpnStatus>(initialVpnStatus);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      const status = await invokeCommand<VpnStatus>("get_vpn_status");
      setVpnStatus((current) => {
        if (current.connectionError && !status.connected && !status.connectionError) {
          return current;
        }
        return status;
      });
      if (status.connected) {
        invokeCommand("subscribe_to_vpn_status").catch((error) =>
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
      const response = await invokeCommand("connect", {
        instanceId: selectedInstance.id,
        region: selectedInstance.region,
        provider: selectedInstance.provider,
        publicIpV4: selectedInstance.publicIpV4 || null,
        publicIpV6: selectedInstance.publicIpV6 || null,
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
    setIsDisconnecting(true);

    try {
      const response = await invokeCommand("disconnect");
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
    } finally {
      setIsDisconnecting(false);
    }
  };

  const clearError = () => {
    setError(null);
  };

  const isDaemonRunning = !vpnStatus.connectionError;

  return {
    vpnStatus,
    checkVpnStatus,
    isConnecting,
    isDisconnecting,
    isDaemonRunning,
    error,
    connectToVpn,
    disconnectFromVpn,
    clearError,
  };
}
