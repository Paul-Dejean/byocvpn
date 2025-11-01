import { useState } from "react";

import "./App.css";
import { VpnPage, SetupPage, LandingPage } from "./pages";

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
