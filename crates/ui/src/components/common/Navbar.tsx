import { Page } from "../../types/pages";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";

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
  const { vpnStatus } = useVpnConnectionContext();
  const isConnected = vpnStatus.connected;

  return (
    <nav className="flex flex-col items-center pt-4 pb-4 gap-2 w-14 bg-gray-800 border-r border-gray-700/50 flex-shrink-0">
      <div className="flex flex-col items-center gap-1 mb-2">
        <div
          className={`w-2 h-2 rounded-full transition-all duration-500 ${
            isConnected ? "bg-green-400 glow-connected-indicator" : "bg-gray-600"
          }`}
        />
        <span
          className={`text-[9px] font-bold tracking-widest uppercase transition-colors duration-500 ${
            isConnected ? "text-green-500" : "text-gray-400"
          }`}
        >
          {isConnected ? "ON" : "OFF"}
        </span>
      </div>

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
      <NavItem
        icon={<SettingsIcon />}
        label="Settings"
        isActive={currentPage === Page.SETTINGS}
        onClick={() => onNavigate(Page.SETTINGS)}
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

function SettingsIcon() {
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
        d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 010-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28z"
      />
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
      />
    </svg>
  );
}
