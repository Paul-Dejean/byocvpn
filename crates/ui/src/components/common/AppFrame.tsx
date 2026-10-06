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
      <main className="h-dvh w-screen bg-bg-strong overflow-hidden">
        <div className="relative h-full w-full flex flex-col bg-bg-strong overflow-hidden text-fg-lighter px-4 safe-area-top">
          {children}
        </div>
      </main>
    );
  }

  return (
    <main className="h-screen w-screen bg-bg-strong overflow-hidden">
      <div
        className="relative flex-shrink-0 bg-bg-strong bg-grid overflow-hidden text-fg-lighter px-4 py-3"
        style={{ width: APP_FRAME_WIDTH, height: APP_FRAME_HEIGHT }}
      >
        {children}
      </div>
    </main>
  );
}
