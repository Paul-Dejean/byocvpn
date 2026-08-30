import { queryOptions } from "@tanstack/react-query";
import { commands } from "../bindings";
import { Instance } from "../types";

export const instancesQueryOptions = queryOptions({
  queryKey: ["instances"],
  queryFn: fetchInstances,
  staleTime: 0,
  refetchOnReconnect: false,
});

async function fetchInstances(): Promise<Instance[]> {
  const result = await commands.listInstances(null);
  if (result.status === "error") {
    throw new Error(result.error);
  }
  return result.data;
}
