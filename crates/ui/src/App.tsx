import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";

import "./App.css";
import "flag-icons/css/flag-icons.min.css";
import {
  ServersPage,
  OnboardingPage,
  SettingsPage,
  PricingPage,
  AddAccountPage,
} from "./pages";
import { AppFrame } from "./components/common/AppFrame";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { Sidebar } from "./components/common/Sidebar";
import { VpnConnectionProvider } from "./contexts/VpnConnectionContext";
import { DeploymentsProvider } from "./contexts/DeploymentsContext";
import { useAutoTerminatedInstanceListener } from "./hooks/useAutoTerminatedInstanceListener";
import { configuredProvidersQueryOptions } from "./queries/configuredProviders";
import { CloudProviderName } from "./types";
import { OnboardingStep } from "./types/onboarding";
import { Page } from "./types/pages";
export { Page };

interface OnboardingState {
  step: OnboardingStep;
  provider: CloudProviderName | null;
}

const INITIAL_ONBOARDING_STATE: OnboardingState = {
  step: OnboardingStep.WELCOME,
  provider: null,
};

function App() {
  const [selectedPage, setSelectedPage] = useState<Page | null>(null);
  const [addAccountProvider, setAddAccountProvider] =
    useState<CloudProviderName | null>(null);
  const [isAddingAccountFromOnboarding, setIsAddingAccountFromOnboarding] =
    useState(false);
  const [onboardingState, setOnboardingState] = useState<OnboardingState>(
    INITIAL_ONBOARDING_STATE,
  );
  const { data: configuredProviders, isError: isConfiguredProvidersError } =
    useQuery(configuredProvidersQueryOptions);
  useAutoTerminatedInstanceListener();

  if (configuredProviders === undefined && !isConfiguredProvidersError) {
    return <AppFrame>{null}</AppFrame>;
  }

  const hasConfiguredProvider =
    configuredProviders !== undefined &&
    configuredProviders.length > 0;
  const page =
    selectedPage ?? (hasConfiguredProvider ? Page.SERVERS : Page.ONBOARDING);
  const setPage = setSelectedPage;

  function openAddAccount(
    provider: CloudProviderName | null,
    fromOnboarding: boolean,
  ) {
    setAddAccountProvider(provider);
    setIsAddingAccountFromOnboarding(fromOnboarding);
    setPage(Page.ADD_ACCOUNT);
  }

  function handleAccountAdded() {
    if (isAddingAccountFromOnboarding && addAccountProvider) {
      setOnboardingState({
        step: OnboardingStep.DEPLOY_SERVER,
        provider: addAccountProvider,
      });
      setPage(Page.ONBOARDING);
      return;
    }
    setPage(Page.SETTINGS);
  }

  function handleAddAccountBack() {
    if (isAddingAccountFromOnboarding) {
      setOnboardingState({
        step: OnboardingStep.CONNECT_ACCOUNT,
        provider: null,
      });
      setPage(Page.ONBOARDING);
      return;
    }
    setPage(Page.SETTINGS);
  }

  return (
    <AppFrame>
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: "var(--color-bg-medium)",
            color: "var(--color-fg-lighter)",
            fontFamily: "var(--font-sans)",
            fontSize: "14px",
            lineHeight: "20px",
            borderRadius: "12px",
            padding: "12px 16px",
          },
          success: {
            iconTheme: {
              primary: "var(--color-fg-success-moderate)",
              secondary: "var(--color-bg-medium)",
            },
          },
          error: {
            iconTheme: {
              primary: "var(--color-fg-danger-moderate)",
              secondary: "var(--color-bg-medium)",
            },
          },
        }}
      />

      <ErrorBoundary>
        <DeploymentsProvider>
          {page === Page.ONBOARDING && (
          <VpnConnectionProvider>
            <OnboardingPage
              key={`${onboardingState.step}-${onboardingState.provider}`}
              initialStep={onboardingState.step}
              provider={onboardingState.provider}
              onSkip={() => setPage(Page.SERVERS)}
              onProviderSelected={(provider) => openAddAccount(provider, true)}
              onFinished={() => setPage(Page.SERVERS)}
            />
          </VpnConnectionProvider>
        )}
        {page === Page.ADD_ACCOUNT && (
          <AddAccountPage
            key={addAccountProvider ?? "select"}
            initialProvider={addAccountProvider}
            onNavigateBack={handleAddAccountBack}
            onAccountAdded={handleAccountAdded}
          />
        )}

        {(page === Page.SERVERS ||
          page === Page.PRICING ||
          page === Page.SETTINGS) && (
          <VpnConnectionProvider>
            <div className="flex h-full gap-4">
              <Sidebar currentPage={page} onNavigate={setPage} />
              <div className="flex-1 min-w-0 h-full overflow-hidden">
                {page === Page.SERVERS && <ServersPage />}
                {page === Page.PRICING && <PricingPage />}
                {page === Page.SETTINGS && (
                  <SettingsPage
                    onNavigateToAddAccount={() => openAddAccount(null, false)}
                  />
                )}
              </div>
            </div>
          </VpnConnectionProvider>
        )}
        </DeploymentsProvider>
      </ErrorBoundary>
    </AppFrame>
  );
}

export default App;
