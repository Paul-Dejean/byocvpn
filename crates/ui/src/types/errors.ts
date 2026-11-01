import type { ErrorKind as ErrorKindBinding, ErrorPayload } from "../bindings";

export const ErrorKind = {
  Authentication: "authentication",
  Authorization: "authorization",
  Quota: "quota",
  Transient: "transient",
  NotFound: "notFound",
  Configuration: "configuration",
  Daemon: "daemon",
  Cloud: "cloud",
  Unknown: "unknown",
} as const satisfies Record<string, ErrorKindBinding>;

export type ErrorKind = ErrorKindBinding;

export type AppError = ErrorPayload;
