import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ServerDetails, ServerStatus } from "../types";

export const useVpnConnection = () => {
  const [serverStatus, setServerStatus] = useState<ServerStatus>("idle");
  const [isConnecting, setIsConnecting] = useState(false);

  const handleConnectToVpn = async (selectedInstance: ServerDetails | null) => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setServerStatus("connecting");

    try {
      const response = await invoke("connect", {
        instanceId: selectedInstance.instance_id,
        region: selectedInstance.region,
      });

      console.log("VPN connected:", response);
      setServerStatus("connected");
    } catch (error) {
      console.error("Failed to connect to VPN:", error);
      setServerStatus("error");
      alert(`Failed to connect to VPN: ${error}`);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnectFromVpn = async () => {
    setServerStatus("idle");

    try {
      const response = await invoke("disconnect");
      console.log("VPN disconnected:", response);
    } catch (error) {
      console.error("Failed to disconnect from VPN:", error);
      alert(`Failed to disconnect from VPN: ${error}`);
    }
  };

  return {
    serverStatus,
    isConnecting,
    setServerStatus,
    handleConnectToVpn,
    handleDisconnectFromVpn,
  };
};
