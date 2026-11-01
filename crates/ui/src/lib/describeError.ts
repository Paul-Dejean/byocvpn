import { AppError, ErrorKind } from "../types";

export interface ErrorDescription {
  title: string;
  description: string;
  hint: string | null;
  detail: string | null;
  summary: string;
}

interface ErrorKindCopy {
  title: string;
  description: string | null;
  hint: string | null;
}

const ERROR_KIND_COPY: Record<ErrorKind, ErrorKindCopy> = {
  [ErrorKind.Authentication]: {
    title: "Credentials rejected",
    description: "The cloud provider did not accept your credentials.",
    hint: "Check the keys for this account in Settings.",
  },
  [ErrorKind.Authorization]: {
    title: "Permission denied",
    description: "Your credentials are missing a permission ByocVPN needs.",
    hint: "Verify the account's permissions in Settings and update its policy.",
  },
  [ErrorKind.Quota]: {
    title: "Quota or rate limit reached",
    description: "The cloud provider is throttling requests or a quota is exhausted.",
    hint: "Wait a moment and try again, or raise the quota in the cloud console.",
  },
  [ErrorKind.Transient]: {
    title: "Temporary failure",
    description: "The operation did not complete in time.",
    hint: "Try again in a moment.",
  },
  [ErrorKind.NotFound]: {
    title: "Resource not found",
    description: null,
    hint: "The resource may have been removed outside ByocVPN. Refresh and try again.",
  },
  [ErrorKind.Configuration]: {
    title: "Configuration problem",
    description: null,
    hint: null,
  },
  [ErrorKind.Daemon]: {
    title: "VPN service unavailable",
    description: "ByocVPN could not talk to its background service.",
    hint: "Make sure the ByocVPN daemon is installed and running.",
  },
  [ErrorKind.Cloud]: {
    title: "Cloud request failed",
    description: null,
    hint: null,
  },
  [ErrorKind.Unknown]: {
    title: "Something went wrong",
    description: null,
    hint: null,
  },
};

export function describeError(error: AppError): ErrorDescription {
  const copy = ERROR_KIND_COPY[error.kind];
  const hasFixedDescription = copy.description !== null;
  const description = copy.description ?? error.message;
  const detailLines = hasFixedDescription ? [error.message, error.detail] : [error.detail];
  const detail = detailLines.filter((line): line is string => Boolean(line)).join("\n");
  const summary = hasFixedDescription ? copy.title : `${copy.title}: ${error.message}`;
  return {
    title: copy.title,
    description,
    hint: copy.hint,
    detail: detail || null,
    summary,
  };
}

export function summarizeError(error: AppError): string {
  return describeError(error).summary;
}
