import { useEffect, useState } from "react";

const MOBILE_LAYOUT_QUERY = "(max-width: 767.98px)";

export function useIsMobileLayout(): boolean {
  const [isMobileLayout, setIsMobileLayout] = useState(
    () => window.matchMedia(MOBILE_LAYOUT_QUERY).matches,
  );

  useEffect(() => {
    const mediaQueryList = window.matchMedia(MOBILE_LAYOUT_QUERY);
    function onLayoutChange(event: MediaQueryListEvent) {
      setIsMobileLayout(event.matches);
    }
    setIsMobileLayout(mediaQueryList.matches);
    mediaQueryList.addEventListener("change", onLayoutChange);
    return () => mediaQueryList.removeEventListener("change", onLayoutChange);
  }, []);

  return isMobileLayout;
}
