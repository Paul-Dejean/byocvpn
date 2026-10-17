import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import App from "./App";

const queryClient = new QueryClient();

void renderApp();

async function renderApp() {
  if (import.meta.env.DEV && !isTauri()) {
    const { installBrowserPreviewMocks } = await import("./dev/browserPreviewMocks");
    installBrowserPreviewMocks();
  }

  const rootElement = document.getElementById("root");
  if (!rootElement) {
    throw new Error("Root element #root not found");
  }

  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </React.StrictMode>,
  );
}
