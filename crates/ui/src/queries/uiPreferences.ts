import { QueryClient, queryOptions } from "@tanstack/react-query";
import { UiPreferences, commands } from "../bindings";

export const uiPreferencesQueryOptions = queryOptions({
  queryKey: ["ui-preferences"],
  queryFn: () => commands.getUiPreferences(),
  staleTime: Infinity,
});

export async function updateUiPreferences(
  queryClient: QueryClient,
  patch: Partial<UiPreferences>,
): Promise<void> {
  const current = await queryClient.ensureQueryData(uiPreferencesQueryOptions);
  const next: UiPreferences = { ...current, ...patch };
  queryClient.setQueryData(uiPreferencesQueryOptions.queryKey, next);
  await commands.saveUiPreferences(next);
}
