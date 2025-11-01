import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Page } from "../App";

export function LandingPage({ setPage }: { setPage: (page: Page) => void }) {
  const [isVisible, setIsVisible] = useState(false);
  const [isCheckingProfile, setIsCheckingProfile] = useState(false);

  useEffect(() => {
    setIsVisible(true);
  }, []);

  const handleGetStarted = async () => {
    setIsCheckingProfile(true);

    try {
      // Check if user already has a profile set up
      const hasProfile = (await invoke("has_profile")) as boolean;

      if (hasProfile) {
        // User has credentials, go directly to VPN page
        setPage(Page.VPN);
      } else {
        // No credentials, go to setup page
        setPage(Page.SETUP);
      }
    } catch (error) {
      console.error("Failed to check profile:", error);
      // Default to setup page on error
      setPage(Page.SETUP);
    } finally {
      setIsCheckingProfile(false);
    }
  };

  return (
    <div className="relative bg-[url('/landing-page-bg.png')] bg-cover bg-center h-screen flex items-center justify-center overflow-hidden">
      {/* Overlay gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-black/50 to-transparent"></div>

      {/* Content container */}
      <div className="container relative z-10 px-4 md:px-0">
        <div className="flex flex-col items-center max-w-3xl mx-auto">
          {/* Logo or icon could go here */}
          <div className="w-16 h-16 rounded-full bg-blue-500/20 backdrop-blur-sm flex items-center justify-center mb-8 border border-blue-400/30">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-8 w-8 text-blue-300"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
              />
            </svg>
          </div>

          {/* Text content with fade-in animation */}
          <div
            className={`flex flex-col text-center transition-opacity duration-1000 ease-in-out ${isVisible ? "opacity-100" : "opacity-0"}`}
          >
            <h1 className="text-5xl font-bold mb-6 text-white bg-clip-text">
              Bring Your Own Cloud VPN
            </h1>

            <p className="text-xl mb-4 text-blue-100">
              Deploy a VPN in your own cloud account for maximum privacy and
              control.
            </p>

            <p className="text-lg mb-10 text-gray-300 max-w-2xl">
              Enjoy full ownership, no third-party access, and the flexibility
              to choose your preferred cloud provider. Your data, your
              infrastructure, your rules.
            </p>

            {/* CTA buttons */}
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <button
                className="px-8 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-all duration-300 transform hover:scale-105 font-medium flex items-center justify-center shadow-lg shadow-blue-600/30 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                onClick={handleGetStarted}
                disabled={isCheckingProfile}
              >
                {isCheckingProfile ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin mr-2"></div>
                    <span>Checking...</span>
                  </>
                ) : (
                  <>
                    <span>Get Started</span>
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="h-5 w-5 ml-2"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fillRule="evenodd"
                        d="M12.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-2.293-2.293a1 1 0 010-1.414z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Feature highlights */}
          <div
            className={`flex flex-wrap justify-center gap-6 mt-16 transition-all duration-1000 delay-500 ${isVisible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-10"}`}
          >
            {["Privacy", "Security", "Control"].map((feature, index) => (
              <div
                key={index}
                className="flex items-center bg-white/10 backdrop-blur-md px-5 py-3 rounded-full"
              >
                <div className="w-2 h-2 rounded-full bg-blue-400 mr-2"></div>
                <span className="text-blue-100 font-medium">{feature}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
