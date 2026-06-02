export interface CommandError {
  command: string;
  message: string;
  timestamp: string;
}

type ErrorListener = () => void;

const errors: CommandError[] = [];
const listeners = new Set<ErrorListener>();

export function pushCommandError(command: string, message: string): void {
  errors.push({ command, message, timestamp: new Date().toLocaleString() });
  listeners.forEach((listener) => listener());
}

export function getCommandErrors(): readonly CommandError[] {
  return errors;
}

export function subscribeToCommandErrors(listener: ErrorListener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
