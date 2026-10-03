import { ReactNode } from "react";

export const APP_FRAME_WIDTH = 1080;
export const APP_FRAME_HEIGHT = 720;

interface AppFrameProps {
  children: ReactNode;
}

export function AppFrame({ children }: AppFrameProps) {
  return (
    <main className="h-screen w-screen bg-gray-800 overflow-hidden">
      <div
        className="relative flex-shrink-0 bg-gray-800 bg-grid overflow-hidden text-primary"
        style={{ width: APP_FRAME_WIDTH, height: APP_FRAME_HEIGHT }}
      >
        {children}
      </div>
    </main>
  );
}
