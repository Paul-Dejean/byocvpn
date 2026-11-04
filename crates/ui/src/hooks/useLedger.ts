import { useQuery } from "@tanstack/react-query";
import { toAppError } from "../lib/appError";
import { summarizeError } from "../lib/describeError";
import { ledgerQueryOptions } from "../queries/ledger";

export function useLedger() {
  const { data, isLoading, error, refetch } = useQuery(ledgerQueryOptions);

  return {
    entries: data ?? [],
    isLoading,
    error: error ? summarizeError(toAppError(error)) : null,
    refetch: () => {
      void refetch();
    },
  };
}
