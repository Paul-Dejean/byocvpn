import { useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { networkLocationQueryOptions } from "../queries/networkLocation";

export function useNetworkLocation(isVpnConnected: boolean) {
  const queryClient = useQueryClient();
  const wasVpnConnected = useRef(isVpnConnected);
  const { data, isLoading, isError } = useQuery({
    ...networkLocationQueryOptions,
    enabled: !isVpnConnected,
  });

  useEffect(() => {
    const hasJustDisconnected = wasVpnConnected.current && !isVpnConnected;
    wasVpnConnected.current = isVpnConnected;
    if (hasJustDisconnected) {
      queryClient.invalidateQueries({
        queryKey: networkLocationQueryOptions.queryKey,
      });
    }
  }, [isVpnConnected, queryClient]);

  return {
    location: data ?? null,
    isLoading,
    isError,
  };
}
