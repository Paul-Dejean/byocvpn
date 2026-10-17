import { useSyncExternalStore } from "react";

const MOBILE_LAYOUT_QUERY = "(max-width: 767.98px)";

export function useIsMobileLayout(): boolean {
  return useSyncExternalStore(subscribeToLayoutChanges, readIsMobileLayout);
}

function subscribeToLayoutChanges(onLayoutChange: () => void): () => void {
  const mediaQueryList = window.matchMedia(MOBILE_LAYOUT_QUERY);
  mediaQueryList.addEventListener("change", onLayoutChange);
  return () => mediaQueryList.removeEventListener("change", onLayoutChange);
}

function readIsMobileLayout(): boolean {
  return window.matchMedia(MOBILE_LAYOUT_QUERY).matches;
}
