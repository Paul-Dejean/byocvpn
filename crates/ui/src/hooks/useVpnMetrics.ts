import { useState, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface VpnMetrics {
  bytesSent: number;
  bytesReceived: number;
  packetsSent: number;
  packetsReceived: number;
  uploadRate: number; // bytes per second
  downloadRate: number; // bytes per second
}

export function useVpnMetrics(isConnected: boolean) {
  const [metrics, setMetrics] = useState<VpnMetrics | null>(null);

  useEffect(() => {
    if (!isConnected) {
      setMetrics(null);
      return;
    }

    let unlisten: UnlistenFn | null = null;

    const setupListener = async () => {
      try {
        // Listen for metrics events from Tauri backend
        unlisten = await listen<VpnMetrics>("vpn-metrics", (event) => {
          setMetrics(event.payload);
        });
      } catch (error) {
        console.error("Failed to setup metrics listener:", error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [isConnected]);

  return metrics;
}
