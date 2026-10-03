import { Server, Settings, Wallet } from "lucide-react";
import { version } from "../../../package.json";
import { Page } from "../../types/pages";
import { NavItem } from "../primitives/NavItem";
import { Logo } from "./Logo";

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <nav className="w-[230px] flex-shrink-0 flex flex-col p-4">
      <div className="w-10 h-10 rounded-lg bg-gray-700 border border-gray-500/60 flex items-center justify-center text-primary">
        <Logo className="w-5 h-5" />
      </div>

      <div className="mt-8 flex flex-col gap-1">
        <NavItem
          icon={<Server size={16} />}
          label="Servers"
          isActive={currentPage === Page.SERVERS}
          onClick={() => onNavigate(Page.SERVERS)}
        />
        <NavItem
          icon={<Wallet size={16} />}
          label="Expenses"
          isActive={currentPage === Page.PRICING}
          onClick={() => onNavigate(Page.PRICING)}
        />
        <NavItem
          icon={<Settings size={16} />}
          label="Settings"
          isActive={currentPage === Page.SETTINGS}
          onClick={() => onNavigate(Page.SETTINGS)}
        />
      </div>

      <span className="mt-auto text-xs text-gray-400">{version}</span>
    </nav>
  );
}
