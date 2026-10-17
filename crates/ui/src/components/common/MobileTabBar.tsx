import { ReactNode } from "react";
import { Server, Settings, Wallet } from "lucide-react";
import { Page } from "../../types/pages";

interface MobileTabBarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

export function MobileTabBar({ currentPage, onNavigate }: MobileTabBarProps) {
  return (
    <nav className="-mx-4 flex shrink-0 items-stretch border-t border-bd-moderate bg-bg-bolder safe-area-bottom">
      <MobileTab
        icon={<Server size={20} />}
        label="Servers"
        isActive={currentPage === Page.SERVERS}
        onClick={() => onNavigate(Page.SERVERS)}
      />
      <MobileTab
        icon={<Wallet size={20} />}
        label="Expenses"
        isActive={currentPage === Page.PRICING}
        onClick={() => onNavigate(Page.PRICING)}
      />
      <MobileTab
        icon={<Settings size={20} />}
        label="Settings"
        isActive={currentPage === Page.SETTINGS}
        onClick={() => onNavigate(Page.SETTINGS)}
      />
    </nav>
  );
}

interface MobileTabProps {
  icon: ReactNode;
  label: string;
  isActive: boolean;
  onClick: () => void;
}

function MobileTab({ icon, label, isActive, onClick }: MobileTabProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={isActive ? "page" : undefined}
      className={`flex flex-1 flex-col items-center gap-1 pt-2 pb-2.5 text-caption transition-colors ${
        isActive ? "text-fg-lighter" : "text-fg-medium"
      }`}
    >
      <span
        className={`flex h-8 w-14 items-center justify-center rounded-full transition-colors ${
          isActive ? "bg-bg-lighter" : ""
        }`}
      >
        {icon}
      </span>
      {label}
    </button>
  );
}
