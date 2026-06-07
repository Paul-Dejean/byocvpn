import { CloudProviderName } from "./providers";

export enum InstanceState {
  Spawning = "SPAWNING",
  Installing = "INSTALLING",
  Error = "ERROR",
  Running = "RUNNING",
  Creating = "CREATING",
  Stopping = "STOPPING",
  Stopped = "STOPPED",
  Deleting = "DELETING",
  Deleted = "DELETED",
  Unknown = "UNKNOWN",
}

export interface Instance {
  id: string;
  name: string;
  state: InstanceState;
  errorReason?: string;
  publicIpV4: string;
  publicIpV6: string;
  region: string;
  provider: CloudProviderName;
}
