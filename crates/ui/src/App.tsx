import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
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
import { useAutoTerminatedInstanceListener } from "./hooks/useAutoTerminatedInstanceListener";
import { configuredProvidersQueryOptions } from "./queries/configuredProviders";
import { Page } from "./types/pages";
export { Page };
function App() {
  const [selectedPage, setSelectedPage] = useState<Page | null>(null);
  const { data: configuredProviders, isError: isConfiguredProvidersError } =
    useQuery(configuredProvidersQueryOptions);
  useAutoTerminatedInstanceListener();

  if (configuredProviders === undefined && !isConfiguredProvidersError) {
    return <main className="bg-grid h-screen" />;
  }

  const hasConfiguredProvider =
    configuredProviders !== undefined && configuredProviders.length > 0;
  const page = selectedPage ?? (hasConfiguredProvider ? Page.VPN : Page.LANDING);
  const setPage = setSelectedPage;

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
              primary: "var(--color-success-500)",
              secondary: "var(--color-gray-800)",
            },
          },
          error: {
            iconTheme: {
              primary: "var(--color-danger-500)",
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
          </VpnConnectionProvider>
        )}
      </ErrorBoundary>
    </main>
  );
}

export default App;
