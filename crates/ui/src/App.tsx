import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { LandingPage } from "./components/LandingPage";
import { SetupPage } from "./components/SetupPage";
import { VpnPage } from "./components/VpnPage";

export enum Page {
  LANDING = "LANDING",
  SETUP = "SETUP",
  VPN = "VPN",
}
function App() {
  const [page, setPage] = useState(Page.LANDING);

  return (
    <main>
      {page === Page.LANDING && <LandingPage setPage={setPage} />}
      {page === Page.SETUP && <SetupPage setPage={setPage} />}
      {page === Page.VPN && <VpnPage />}
    </main>
  );
}

export default App;
