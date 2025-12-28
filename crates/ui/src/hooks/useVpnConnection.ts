import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";
import { ServerDetails, ServerStatus } from "../types";

export const useVpnConnection = () => {
  const [serverStatus, setServerStatus] = useState<ServerStatus>("idle");
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnectToVpn = async (selectedInstance: ServerDetails | null) => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setServerStatus("connecting");
    setError(null);

    try {
      const response = await invoke("connect", {
        instanceId: selectedInstance.instance_id,
        region: selectedInstance.region,
      });

      console.log("VPN connected:", response);
      console.log("Setting serverStatus to 'connected'");
      setServerStatus("connected");
      toast.success("Connected to VPN successfully!");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to connect to VPN";
      setError(errorMessage);
      console.error("Failed to connect to VPN:", error);
      setServerStatus("error");
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnectFromVpn = async () => {
    setServerStatus("idle");
    setError(null);

    try {
      const response = await invoke("disconnect");
      console.log("VPN disconnected:", response);
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
    serverStatus,
    isConnecting,
    error,
    setServerStatus,
    handleConnectToVpn,
    handleDisconnectFromVpn,
    clearError,
  };
};
