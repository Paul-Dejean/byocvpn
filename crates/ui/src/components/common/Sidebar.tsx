import { Server, Settings, Wallet } from "lucide-react";
import { version } from "../../../package.json";
import { Page } from "../../types/pages";
import { NavItem } from "../primitives/NavItem";
import { Logo } from "./Logo";

export const SIDEBAR_WIDTH = 198;

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <nav
      className="flex-shrink-0 flex flex-col justify-between"
      style={{ width: SIDEBAR_WIDTH }}
    >
      <div className="flex flex-col gap-6">
        <div className="w-[42px] h-[42px] rounded-lg bg-bg-bolder border border-bd-moderate flex items-center justify-center text-fg-lighter">
          <Logo className="w-6 h-6" />
        </div>

        <div className="flex flex-col gap-2">
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
      </div>

      <span className="text-caption text-fg-moderate">{version}</span>
    </nav>
  );
}
