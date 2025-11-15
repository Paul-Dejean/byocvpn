import { useState } from "react";
import { Toaster } from "react-hot-toast";

import "./App.css";
import { VpnPage, SetupPage, LandingPage, SettingsPage } from "./pages";

export enum Page {
  LANDING = "LANDING",
  SETUP = "SETUP",
  VPN = "VPN",
  SETTINGS = "SETTINGS",
}
function App() {
  const [page, setPage] = useState(Page.LANDING);

  return (
    <main>
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
      {page === Page.SETUP && <SetupPage setPage={setPage} />}
      {page === Page.VPN && (
        <VpnPage onNavigateToSettings={() => setPage(Page.SETTINGS)} />
      )}
      {page === Page.SETTINGS && (
        <SettingsPage onNavigateBack={() => setPage(Page.VPN)} />
      )}
    </main>
  );
}

export default App;
