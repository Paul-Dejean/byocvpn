import { ReactNode } from "react";
import { useIsMobileLayout } from "../../hooks/useIsMobileLayout";
import { Page } from "../../types/pages";
import { MobileTabBar } from "./MobileTabBar";
import { Sidebar } from "./Sidebar";

interface MainLayoutProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  children: ReactNode;
}

export function MainLayout({ currentPage, onNavigate, children }: MainLayoutProps) {
  const isMobileLayout = useIsMobileLayout();

  if (isMobileLayout) {
    return (
      <div className="flex-1 min-h-0 flex flex-col">
        <div className="flex-1 min-h-0 overflow-hidden pb-3">{children}</div>
        <MobileTabBar currentPage={currentPage} onNavigate={onNavigate} />
      </div>
    );
  }

  return (
    <div className="flex h-full gap-4">
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <div className="flex-1 min-w-0 h-full overflow-hidden">{children}</div>
    </div>
  );
}
