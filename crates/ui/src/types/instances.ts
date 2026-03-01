export interface Instance {
  id: string;
  name: string;
  state: string;
  publicIpV4: string;
  publicIpV6: string;
  region: string;
  provider: string;
}

export type ServerStatus =
  | "idle"
  | "spawning"
  | "running"
  | "error"
  | "terminating"
  | "connecting"
  | "connected";
