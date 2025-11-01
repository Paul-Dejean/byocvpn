export interface ExistingInstance {
  id: string;
  name?: string;
  state: string;
  public_ip_v4: string;
  public_ip_v6: string;
  region?: string;
}

export interface ServerDetails {
  instance_id: string;
  public_ip_v4: string;
  public_ip_v6?: string;
  region: string;
  client_private_key: string;
  server_public_key: string;
}

export type ServerStatus =
  | "idle"
  | "spawning"
  | "running"
  | "error"
  | "terminating"
  | "connecting"
  | "connected";
