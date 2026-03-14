import { Page } from "../../types/pages";
interface NavbarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

interface NavItemProps {
  icon: React.ReactNode;
  label: string;
  isActive: boolean;
  onClick: () => void;
}

function NavItem({ icon, label, isActive, onClick }: NavItemProps) {
  return (
    <button
      onClick={onClick}
      title={label}
      className={`flex flex-col items-center justify-center w-10 h-10 rounded-lg transition-colors ${
        isActive
          ? "bg-blue-600 text-white"
          : "text-gray-400 hover:bg-gray-700 hover:text-white"
      }`}
    >
      {icon}
    </button>
  );
}

export function Navbar({ currentPage, onNavigate }: NavbarProps) {
  return (
    <nav className="flex flex-col items-center pt-4 pb-4 gap-2 w-14 bg-gray-800 border-r border-gray-700 flex-shrink-0">
      <NavItem
        icon={<ServerIcon />}
        label="Servers"
        isActive={currentPage === Page.VPN}
        onClick={() => onNavigate(Page.VPN)}
      />
      <NavItem
        icon={<DollarIcon />}
        label="Expenses"
        isActive={currentPage === Page.PRICING}
        onClick={() => onNavigate(Page.PRICING)}
      />
    </nav>
  );
}

function ServerIcon() {
  return (
    <svg
      className="w-5 h-5"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
      />
    </svg>
  );
}

function DollarIcon() {
  return (
    <svg
      className="w-5 h-5"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
      />
    </svg>
  );
}
