import { queryOptions } from "@tanstack/react-query";

const NETWORK_LOCATION_ENDPOINT = "https://ipapi.co/json/";
const NETWORK_LOCATION_STALE_TIME_MS = 5 * 60 * 1000;

export interface NetworkLocation {
  publicIp: string;
  city: string;
  country: string;
  countryCode: string;
}

interface NetworkLocationResponse {
  ip?: string;
  city?: string;
  country_name?: string;
  country_code?: string;
}

export const networkLocationQueryOptions = queryOptions({
  queryKey: ["network-location"],
  queryFn: fetchNetworkLocation,
  staleTime: NETWORK_LOCATION_STALE_TIME_MS,
  retry: 1,
});

async function fetchNetworkLocation(): Promise<NetworkLocation> {
  const response = await fetch(NETWORK_LOCATION_ENDPOINT);
  if (!response.ok) {
    throw new Error(`Location lookup failed with status ${response.status}`);
  }
  const payload: NetworkLocationResponse = await response.json();
  return {
    publicIp: payload.ip ?? "",
    city: payload.city ?? "",
    country: payload.country_name ?? "",
    countryCode: (payload.country_code ?? "").toLowerCase(),
  };
}
