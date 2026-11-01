import { AppError, ErrorKind } from "../types";

export class AppErrorException extends Error {
  readonly appError: AppError;

  constructor(appError: AppError) {
    super(appError.message);
    this.name = "AppErrorException";
    this.appError = appError;
  }
}

export function toAppError(value: unknown): AppError {
  if (value instanceof AppErrorException) {
    return value.appError;
  }
  if (isAppError(value)) {
    return value;
  }
  const message = value instanceof Error ? value.message : String(value);
  return { kind: ErrorKind.Unknown, message, detail: null };
}

function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "kind" in value &&
    "message" in value &&
    typeof value.message === "string"
  );
}
