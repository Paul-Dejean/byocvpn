import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export const useDaemonInstaller = () => {
  const [isChecking, setIsChecking] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkDaemonInstalled = async (): Promise<boolean> => {
    setIsChecking(true);
    setError(null);
    try {
      return await invoke<boolean>("is_daemon_installed");
    } catch (err) {
      setError("Failed to check daemon installation status.");
      console.error("Failed to check daemon installation:", err);
      return false;
    } finally {
      setIsChecking(false);
    }
  };

  const installDaemon = async (): Promise<boolean> => {
    setIsInstalling(true);
    setError(null);
    try {
      await invoke("install_daemon");
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);

      if (errorMessage.includes("User cancelled")) {
        return false;
      }

      setError(errorMessage);
      console.error("Failed to install daemon:", err);
      return false;
    } finally {
      setIsInstalling(false);
    }
  };

  const clearError = () => setError(null);

  return {
    isChecking,
    isInstalling,
    error,
    checkDaemonInstalled,
    installDaemon,
    clearError,
  };
};
