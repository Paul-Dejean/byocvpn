import { useState } from "react";
import { Toaster } from "react-hot-toast";

import "./App.css";
import {
  VpnPage,
  SetupPage,
  LandingPage,
  SettingsPage,
  DaemonSetupPage,
  PricingPage,
} from "./pages";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { Navbar } from "./components/common/Navbar";
import { Page } from "./types/pages";
export { Page };
function App() {
  const [page, setPage] = useState(Page.LANDING);

  return (
    <main>
      <ErrorBoundary>
        <Toaster
          position="top-right"
          toastOptions={{
            duration: 4000,
            style: {
              background: "#1f2937",
              color: "#fff",
              border: "1px solid #374151",
            },
            success: {
              iconTheme: {
                primary: "#10b981",
                secondary: "#fff",
              },
            },
            error: {
              iconTheme: {
                primary: "#ef4444",
                secondary: "#fff",
              },
            },
          }}
        />
        {page === Page.LANDING && <LandingPage setPage={setPage} />}
        {page === Page.DAEMON_SETUP && <DaemonSetupPage setPage={setPage} />}
        {page === Page.SETUP && <SetupPage setPage={setPage} />}
        {page === Page.SETTINGS && (
          <SettingsPage onNavigateBack={() => setPage(Page.VPN)} />
        )}

        {}
        {(page === Page.VPN || page === Page.PRICING) && (
          <div className="flex h-screen">
            <Navbar currentPage={page} onNavigate={setPage} />
            <div className="flex-1 min-w-0 overflow-hidden">
              {page === Page.VPN && (
                <VpnPage onNavigateToSettings={() => setPage(Page.SETTINGS)} />
              )}
              {page === Page.PRICING && <PricingPage />}
            </div>
          </div>
        )}
      </ErrorBoundary>
    </main>
  );
}

export default App;
