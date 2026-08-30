import type {
  InstanceInfo,
  InstanceState as InstanceStateBinding,
} from "../bindings";

export const InstanceState = {
  Spawning: "SPAWNING",
  Installing: "INSTALLING",
  Error: "ERROR",
  Running: "RUNNING",
  Stopping: "STOPPING",
  Stopped: "STOPPED",
  Unknown: "UNKNOWN",
} as const satisfies Record<string, InstanceStateBinding>;

export type InstanceState = InstanceStateBinding;

export type Instance = InstanceInfo;
