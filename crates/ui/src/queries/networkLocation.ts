import { queryOptions } from "@tanstack/react-query";
import { getCountryName } from "../lib/countryName";

const NETWORK_LOCATION_ENDPOINT = "https://ipinfo.io/json";
const NETWORK_LOCATION_STALE_TIME_MS = 10 * 60 * 1000;

export interface NetworkLocation {
  publicIp: string;
  city: string;
  country: string;
  countryCode: string;
}

interface NetworkLocationResponse {
  ip?: string;
  city?: string;
  country?: string;
}

export const networkLocationQueryOptions = queryOptions({
  queryKey: ["network-location"],
  queryFn: fetchNetworkLocation,
  staleTime: NETWORK_LOCATION_STALE_TIME_MS,
  retry: 1,
});

async function fetchNetworkLocation(): Promise<NetworkLocation> {
  const response = await fetch(NETWORK_LOCATION_ENDPOINT, {
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    throw new Error(`Location lookup failed with status ${response.status}`);
  }
  const payload: NetworkLocationResponse = await response.json();
  const countryCode = (payload.country ?? "").toLowerCase();
  return {
    publicIp: payload.ip ?? "",
    city: payload.city ?? "",
    country: getCountryName(countryCode),
    countryCode,
  };
}
