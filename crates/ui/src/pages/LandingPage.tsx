import { useState, useEffect } from "react";
import { Page } from "../types/pages";

export function LandingPage({ setPage }: { setPage: (page: Page) => void }) {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    setIsVisible(true);
  }, []);

  const handleGetStarted = () => {
    setPage(Page.VPN);
  };

  return (
    <div className="relative bg-[url('/landing-page-bg.png')] bg-cover bg-center h-screen flex items-center justify-center overflow-hidden">
      {}
      <div className="absolute inset-0 bg-gradient-to-br from-black/50 to-transparent"></div>

      {}
      <div className="container relative z-10 px-4 md:px-0">
        <div className="flex flex-col items-center max-w-3xl mx-auto">
          <div className="relative flex items-center justify-center mb-8">
            <div
              className="absolute w-44 h-44 rounded-full border border-blue-400/10 animate-ping"
              style={{ animationDuration: "4s" }}
            />
            <div
              className="absolute w-32 h-32 rounded-full border border-blue-400/15 animate-ping"
              style={{ animationDuration: "3s", animationDelay: "0.75s" }}
            />
            <div
              className="absolute w-24 h-24 rounded-full border border-blue-400/25 animate-ping"
              style={{ animationDuration: "2.5s", animationDelay: "1.25s" }}
            />
            <div
              className="absolute w-20 h-20 rounded-full border border-blue-400/30"
              style={{ boxShadow: "0 0 24px rgba(32, 180, 250, 0.12)" }}
            />
            <div
              className="w-16 h-16 rounded-full bg-blue-500/20 backdrop-blur-sm flex items-center justify-center relative z-10 border border-blue-400/30"
              style={{ boxShadow: "0 0 32px rgba(32, 180, 250, 0.2)" }}
            >
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
          </div>

          {}
          <div
            className={`flex flex-col text-center transition-opacity duration-1000 ease-in-out ${isVisible ? "opacity-100" : "opacity-0"}`}
          >
            <h1 className="text-5xl font-bold mb-6 text-gradient">
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

            {}
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <button
                className="px-8 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-all duration-300 transform hover:scale-105 font-medium flex items-center justify-center shadow-lg shadow-blue-600/30"
                onClick={handleGetStarted}
              >
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
              </button>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
