import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toAppError } from "../lib/appError";
import { summarizeError } from "../lib/describeError";
import { ledgerQueryOptions } from "../queries/ledger";

export function useLedger() {
  const queryClient = useQueryClient();
  const { data, isLoading, error, refetch } = useQuery(ledgerQueryOptions(queryClient));

  return {
    entries: data ?? [],
    isLoading,
    error: error ? summarizeError(toAppError(error)) : null,
    refetch: () => {
      void refetch();
    },
  };
}
