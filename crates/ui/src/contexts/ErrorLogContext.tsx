import { createContext, useContext, useState, ReactNode } from "react";

export interface ErrorLogEntry {
  id: string;
  timestamp: Date;
  message: string;
  source: string;
}

interface ErrorLogContextValue {
  entries: ErrorLogEntry[];
  addEntry: (message: string, source: string) => void;
  clearEntries: () => void;
}

const ErrorLogContext = createContext<ErrorLogContextValue | null>(null);

interface ErrorLogProviderProps {
  children: ReactNode;
}

export function ErrorLogProvider({ children }: ErrorLogProviderProps) {
  const [entries, setEntries] = useState<ErrorLogEntry[]>([]);

  const addEntry = (message: string, source: string) => {
    setEntries((previous) => [
      { id: `${Date.now()}-${Math.random()}`, timestamp: new Date(), message, source },
      ...previous,
    ]);
  };

  const clearEntries = () => setEntries([]);

  return (
    <ErrorLogContext.Provider value={{ entries, addEntry, clearEntries }}>
      {children}
    </ErrorLogContext.Provider>
  );
}

export function useErrorLogContext() {
  const context = useContext(ErrorLogContext);
  if (!context) {
    throw new Error("useErrorLogContext must be used within ErrorLogProvider");
  }
  return context;
}
