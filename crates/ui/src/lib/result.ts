export type Result<T, E = string> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export function ok<T>(data: T): Result<T, never> {
  return { status: "ok", data };
}

export function err<E>(error: E): Result<never, E> {
  return { status: "error", error };
}

export async function fromPromise<T>(
  promise: Promise<T>,
): Promise<Result<T, string>> {
  try {
    return ok(await promise);
  } catch (error) {
    return err(error instanceof Error ? error.message : String(error));
  }
}
