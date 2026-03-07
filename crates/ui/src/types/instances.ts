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

// ── Spawn pipeline types ─────────────────────────────────────────────────────

export type SpawnStepStatus = "pending" | "running" | "completed" | "failed";

/** A single deployment step as returned by the backend's spawn_steps(). */
export interface SpawnStep {
  id: string;
  label: string;
}

/** Returned immediately by the spawn_instance Tauri command. */
export interface SpawnJob {
  jobId: string;
  steps: SpawnStep[];
  region: string;
  provider: string;
}

/** A step enriched with its current execution status (frontend-only). */
export interface SpawnStepState extends SpawnStep {
  status: SpawnStepStatus;
  error?: string;
}

/** Payload of the "spawn-progress" Tauri event. */
export interface SpawnProgressEvent {
  jobId: string;
  stepId: string;
  status: SpawnStepStatus;
  error?: string;
}

/** Payload of the "spawn-complete" Tauri event. */
export interface SpawnCompleteEvent {
  jobId: string;
  instance: Instance;
}

/** All step states for one in-flight spawn job (frontend-only). */
export interface SpawnJobState {
  jobId: string;
  steps: SpawnStepState[];
}
