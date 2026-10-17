import { useState, useEffect, type Dispatch, type SetStateAction } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "react-hot-toast";
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

  function checkVpnStatus(): Promise<void> {
    return refreshVpnStatus(setVpnStatus);
  }

  useEffect(() => {
    void refreshVpnStatus(setVpnStatus);

    const unlistenPromise = events.vpnStatus.listen((event) => {
      setVpnStatus(event.payload);
    });

    const unlistenFocusPromise = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) void refreshVpnStatus(setVpnStatus);
    });

    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
      void unlistenFocusPromise.then((unlisten) => unlisten());
    };
  }, []);

  const connectToVpn = async (selectedInstance: Instance) => {
    if (!selectedInstance) return;

    setIsConnecting(true);
    setError(null);

    if (vpnStatus.connected) {
      const disconnectResult = await commands.disconnect();
      if (disconnectResult.status === "error") {
        setIsConnecting(false);
        setError(disconnectResult.error);
        toast.error(disconnectResult.error);
        return;
      }
    }

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

  const disconnectFromVpn = async (): Promise<boolean> => {
    setError(null);
    setIsDisconnecting(true);

    const result = await commands.disconnect();
    setIsDisconnecting(false);
    if (result.status === "error") {
      setError(result.error);
      toast.error(result.error);
      console.error("Failed to disconnect from VPN:", result.error);
      return false;
    }
    console.log("VPN disconnected:", result.data);
    return true;
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

async function refreshVpnStatus(setVpnStatus: Dispatch<SetStateAction<VpnStatus>>): Promise<void> {
  const result = await commands.getVpnStatus();
  if (result.status === "error") {
    console.error("Failed to check VPN status:", result.error);
    return;
  }
  const status = result.data;
  setVpnStatus((current) => mergeVpnStatus(current, status));
  if (status.connected) {
    const subscribeResult = await commands.subscribeToVpnStatus();
    if (subscribeResult.status === "error") {
      console.error("Failed to resume metrics stream:", subscribeResult.error);
    }
  }
}

function mergeVpnStatus(current: VpnStatus, status: VpnStatus): VpnStatus {
  if (current.connectionError && !status.connected && !status.connectionError) {
    return current;
  }
  return status;
}
