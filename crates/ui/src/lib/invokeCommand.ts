import { invoke } from "@tauri-apps/api/core";
import { pushCommandError } from "./commandErrorStore";

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    pushCommandError(command, error as string);
    throw new Error(error as string);
  }
}
