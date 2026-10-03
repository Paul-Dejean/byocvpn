import { queryOptions } from "@tanstack/react-query";
import { getCountryName } from "../lib/countryName";

const LOCATION_ENDPOINT = "https://ipinfo.io/json";
const IPV4_ENDPOINT = "https://ipv4.icanhazip.com";
const IPV6_ENDPOINT = "https://ipv6.icanhazip.com";
const NETWORK_LOCATION_STALE_TIME_MS = 10 * 60 * 1000;

export interface NetworkLocation {
  publicIpV4: string;
  publicIpV6: string;
  city: string;
  country: string;
  countryCode: string;
}

interface LocationResponse {
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
  const [locationResult, ipV4Result, ipV6Result] = await Promise.allSettled([
    fetchLocation(),
    fetchPublicIp(IPV4_ENDPOINT),
    fetchPublicIp(IPV6_ENDPOINT),
  ]);

  const location =
    locationResult.status === "fulfilled" ? locationResult.value : null;
  const publicIpV4 = ipV4Result.status === "fulfilled" ? ipV4Result.value : "";
  const publicIpV6 = ipV6Result.status === "fulfilled" ? ipV6Result.value : "";

  if (location === null && !publicIpV4 && !publicIpV6) {
    throw new Error("Network location lookup failed");
  }

  const countryCode = (location?.country ?? "").toLowerCase();
  return {
    publicIpV4,
    publicIpV6,
    city: location?.city ?? "",
    country: getCountryName(countryCode),
    countryCode,
  };
}

async function fetchLocation(): Promise<LocationResponse> {
  const response = await fetch(LOCATION_ENDPOINT, {
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    throw new Error(`Location lookup failed with status ${response.status}`);
  }
  return response.json();
}

async function fetchPublicIp(endpoint: string): Promise<string> {
  const response = await fetch(endpoint);
  if (!response.ok) {
    throw new Error(`IP lookup failed with status ${response.status}`);
  }
  return (await response.text()).trim();
}
