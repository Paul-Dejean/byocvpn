import { useState } from "react";

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
