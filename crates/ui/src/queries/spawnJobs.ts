import { queryOptions } from "@tanstack/react-query";
import { commands } from "../bindings";
import { SpawnJob } from "../types";

const RUNNING_JOB_POLL_INTERVAL_MS = 3000;

export const spawnJobsQueryOptions = queryOptions({
  queryKey: ["spawn-jobs"],
  queryFn: fetchActiveSpawnJobs,
  staleTime: 0,
  refetchOnReconnect: false,
  refetchInterval: (query) =>
    query.state.data?.length ? RUNNING_JOB_POLL_INTERVAL_MS : false,
});

async function fetchActiveSpawnJobs(): Promise<SpawnJob[]> {
  const result = await commands.listActiveSpawnJobs();
  if (result.status === "error") {
    throw new Error(result.error);
  }
  return result.data;
}
