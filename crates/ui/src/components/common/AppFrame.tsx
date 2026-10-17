import { ReactNode } from "react";
import { useIsMobileLayout } from "../../hooks/useIsMobileLayout";

export const APP_FRAME_WIDTH = 1080;
export const APP_FRAME_HEIGHT = 720;

interface AppFrameProps {
  children: ReactNode;
}

export function AppFrame({ children }: AppFrameProps) {
  const isMobileLayout = useIsMobileLayout();

  if (isMobileLayout) {
    return (
      <main className="h-dvh w-screen overflow-hidden bg-bg-strong">
        <div className="relative flex size-full flex-col overflow-hidden bg-bg-strong px-4 safe-area-top text-fg-lighter">
          {children}
        </div>
      </main>
    );
  }

  return (
    <main className="h-screen w-screen overflow-hidden bg-bg-strong">
      <div
        className="relative shrink-0 overflow-hidden bg-bg-strong bg-grid px-4 py-3 text-fg-lighter"
        style={{ width: APP_FRAME_WIDTH, height: APP_FRAME_HEIGHT }}
      >
        {children}
      </div>
    </main>
  );
}
