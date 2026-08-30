import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import toast from "react-hot-toast";
import { commands, events } from "../bindings";
import { Instance, VpnStatus } from "../types";

const initialVpnStatus: VpnStatus = {
  connected: false,
  instance: null,
  metrics: null,
  connectedAt: null,
  connectionError: null,
};

export function useVpnConnection() {
  const [vpnStatus, setVpnStatus] = useState<VpnStatus>(initialVpnStatus);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkVpnStatus();

    const unlistenPromise = events.vpnStatus.listen((event) => {
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
    const result = await commands.getVpnStatus();
    if (result.status === "error") {
      console.error("Failed to check VPN status:", result.error);
      return;
    }
    const status = result.data;
    setVpnStatus((current) => {
      if (current.connectionError && !status.connected && !status.connectionError) {
        return current;
      }
      return status;
    });
    if (status.connected) {
      const subscribeResult = await commands.subscribeToVpnStatus();
      if (subscribeResult.status === "error") {
        console.error(
          "Failed to resume metrics stream:",
          subscribeResult.error,
        );
      }
    }
  };

  const connectToVpn = async (selectedInstance: Instance) => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setError(null);

    const result = await commands.connect(
      selectedInstance.id,
      selectedInstance.region,
      selectedInstance.provider,
      selectedInstance.publicIpV4 || null,
      selectedInstance.publicIpV6 || null,
    );
    setIsConnecting(false);
    if (result.status === "error") {
      setError(result.error);
      console.error("Failed to connect to VPN:", result.error);
      toast.error(result.error);
      return;
    }
    console.log("VPN connected:", result.data);
  };

  const disconnectFromVpn = async () => {
    setError(null);
    setIsDisconnecting(true);

    const result = await commands.disconnect();
    setIsDisconnecting(false);
    if (result.status === "error") {
      setError(result.error);
      toast.error(result.error);
      console.error("Failed to disconnect from VPN:", result.error);
      return;
    }
    console.log("VPN disconnected:", result.data);
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
