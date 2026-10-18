import { ReactNode } from "react";
import { useIsMobileLayout } from "../../hooks/useIsMobileLayout";
import { Page } from "../../types/pages";
import { MobileTabBar } from "./MobileTabBar";
import { Sidebar } from "./Sidebar";
import { UpdateBanner } from "./UpdateBanner";

interface MainLayoutProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  children: ReactNode;
}

export function MainLayout({ currentPage, onNavigate, children }: MainLayoutProps) {
  const isMobileLayout = useIsMobileLayout();

  if (isMobileLayout) {
    return (
      <div className="flex min-h-0 flex-1 flex-col">
        <UpdateBanner />
        <div className="min-h-0 flex-1 overflow-hidden pb-3">{children}</div>
        <MobileTabBar currentPage={currentPage} onNavigate={onNavigate} />
      </div>
    );
  }

  return (
    <div className="flex h-full gap-4">
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <div className="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
        <UpdateBanner />
        <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
      </div>
    </div>
  );
}
