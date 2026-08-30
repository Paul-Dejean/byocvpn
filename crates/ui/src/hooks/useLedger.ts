import { useQuery, useQueryClient } from "@tanstack/react-query";
import { extractErrorMessage } from "../lib/extractErrorMessage";
import { ledgerQueryOptions } from "../queries/ledger";

export function useLedger() {
  const queryClient = useQueryClient();
  const { data, isLoading, error, refetch } = useQuery(
    ledgerQueryOptions(queryClient),
  );

  return {
    entries: data ?? [],
    isLoading,
    error: error
      ? extractErrorMessage(error, "Failed to load cost ledger")
      : null,
    refetch: () => {
      refetch();
    },
  };
}
