import { ReactNode } from "react";

export const APP_FRAME_WIDTH = 1080;
export const APP_FRAME_HEIGHT = 720;

interface AppFrameProps {
  children: ReactNode;
}

export function AppFrame({ children }: AppFrameProps) {
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
