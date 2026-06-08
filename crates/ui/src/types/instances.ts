import { CloudProviderName } from "./providers";

export enum InstanceState {
  Spawning = "SPAWNING",
  Installing = "INSTALLING",
  Error = "ERROR",
  Running = "RUNNING",
  Stopping = "STOPPING",
  Stopped = "STOPPED",
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
  spawnId?: string;
  instanceType: string;
  launchedAt: string;
}
