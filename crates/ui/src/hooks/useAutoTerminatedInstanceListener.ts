import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { events } from "../bindings";
import { instancesQueryOptions } from "../queries/instances";

export function useAutoTerminatedInstanceListener() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlistenPromise = events.instanceAutoTerminated.listen(() => {
      queryClient.invalidateQueries({
        queryKey: instancesQueryOptions.queryKey,
      });
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);
}
