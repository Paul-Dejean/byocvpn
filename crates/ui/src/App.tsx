import { useState } from "react";
import { Toaster } from "react-hot-toast";

import "./App.css";
import "flag-icons/css/flag-icons.min.css";
import {
  VpnPage,
  LandingPage,
  SettingsPage,
  PricingPage,
  AddAccountPage,
} from "./pages";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { Navbar } from "./components/common/Navbar";
import { VpnConnectionProvider } from "./contexts/VpnConnectionContext";
import { RegionsProvider, InstancesProvider } from "./contexts";
import { Page } from "./types/pages";
export { Page };
function App() {
  const [page, setPage] = useState(Page.LANDING);

  return (
    <main className="bg-grid">
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: "var(--color-gray-800)",
            color: "var(--color-gray-100)",
            border: "1px solid var(--color-gray-500)",
            fontFamily: "var(--font-sans)",
          },
          success: {
            iconTheme: {
              primary: "#10b981",
              secondary: "var(--color-gray-800)",
            },
          },
          error: {
            iconTheme: {
              primary: "#ef4444",
              secondary: "var(--color-gray-800)",
            },
          },
        }}
      />

      <ErrorBoundary>
        {page === Page.LANDING && <LandingPage setPage={setPage} />}
        {page === Page.ADD_ACCOUNT && (
          <AddAccountPage
            onNavigateBack={() => setPage(Page.VPN)}
            onAccountAdded={() => setPage(Page.VPN)}
          />
        )}

        {(page === Page.VPN ||
          page === Page.PRICING ||
          page === Page.SETTINGS) && (
          <VpnConnectionProvider>
            <RegionsProvider>
              <InstancesProvider>
                <div className="flex h-screen">
                  <Navbar currentPage={page} onNavigate={setPage} />
                  <div className="flex-1 min-w-0 overflow-hidden">
                    {page === Page.VPN && <VpnPage />}
                    {page === Page.PRICING && <PricingPage />}
                    {page === Page.SETTINGS && (
                      <SettingsPage
                        onNavigateToAddAccount={() => setPage(Page.ADD_ACCOUNT)}
                      />
                    )}
                  </div>
                </div>
              </InstancesProvider>
            </RegionsProvider>
          </VpnConnectionProvider>
        )}
      </ErrorBoundary>
    </main>
  );
}

export default App;
