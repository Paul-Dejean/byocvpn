import { SpawnStepState, SpawnStepStatus } from "../../types";

interface ProvisionAccountDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  provider: string;
  title?: string;
  identifier?: string;
  subtitle?: string;
  steps: SpawnStepState[];
  isComplete: boolean;
  successMessage?: string;
  error: string | null;
}

const StepStatusIcon = ({ status }: { status: SpawnStepStatus }) => {
  switch (status) {
    case "running":
      return (
        <div className="w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full animate-spin flex-shrink-0" />
      );
    case "completed":
      return (
        <svg
          className="w-5 h-5 text-green-400 flex-shrink-0"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M5 13l4 4L19 7"
          />
        </svg>
      );
    case "failed":
      return (
        <svg
          className="w-5 h-5 text-red-400 flex-shrink-0"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      );
    default:
      return (
        <div className="w-5 h-5 rounded-full border-2 border-gray-600 flex-shrink-0" />
      );
  }
};

const formatProviderLabel = (provider: string): string => {
  switch (provider) {
    case "aws":
      return "AWS";
    case "azure":
      return "Azure";
    case "gcp":
      return "Google Cloud";
    case "oracle":
      return "Oracle Cloud";
    default:
      return provider;
  }
};

export function ProvisionAccountDrawer({
  isOpen,
  onClose,
  provider,
  title,
  identifier,
  subtitle,
  steps,
  isComplete,
  successMessage,
  error,
}: ProvisionAccountDrawerProps) {
  return (
    <>
      <div
        className={`fixed inset-0 z-40 bg-black transition-opacity duration-300 ${
          isOpen
            ? "opacity-50 pointer-events-auto"
            : "opacity-0 pointer-events-none"
        }`}
        onClick={onClose}
      />

      <div
        className={`fixed top-0 right-0 h-full w-96 bg-gray-800 z-50 flex flex-col shadow-2xl border-l border-gray-700/50 transition-transform duration-300 ease-in-out ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <div className="flex items-center justify-between p-6 border-b border-gray-700/50 flex-shrink-0">
          <div>
            <h2 className="text-lg font-semibold text-white">
              {title ?? `Provisioning ${formatProviderLabel(provider)}`}
            </h2>
            {identifier && (
              <p className="text-xs text-gray-500 font-mono mt-0.5">{identifier}</p>
            )}
            <p className="text-sm text-gray-400 mt-1">
              {subtitle ?? "Setting up your account infrastructure"}
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg transition-colors text-gray-400 hover:text-white hover:bg-gray-600/50"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          <div className="space-y-4">
            {steps.map((step, index) => (
              <div key={step.id} className="flex items-start gap-3">
                <div className="mt-0.5">
                  <StepStatusIcon status={step.status} />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-gray-500 font-mono tabular-nums">
                      {String(index + 1).padStart(2, "0")}
                    </span>
                    <span
                      className={`text-sm font-medium ${
                        step.status === "completed"
                          ? "text-gray-300"
                          : step.status === "running"
                            ? "text-white"
                            : step.status === "failed"
                              ? "text-red-300"
                              : "text-gray-500"
                      }`}
                    >
                      {step.label}
                    </span>
                  </div>
                  {step.error && (
                    <p className="text-xs text-red-400 mt-1 ml-6">
                      {step.error}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="p-6 border-t border-gray-700/50 flex-shrink-0">
          {isComplete && (
            <div className="flex items-center gap-3 p-4 mb-4 bg-green-900/50 border border-green-700 rounded-lg">
              <svg
                className="w-5 h-5 text-green-400 flex-shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 13l4 4L19 7"
                />
              </svg>
              <p className="text-green-300 text-sm font-medium">
                {successMessage ?? "Account provisioned successfully"}
              </p>
            </div>
          )}
          {error && (
            <div className="flex items-start gap-3 p-4 mb-4 bg-red-900/50 border border-red-700 rounded-lg">
              <svg
                className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
              <div>
                <p className="text-red-300 text-sm font-medium">Provisioning failed</p>
                <p className="text-xs text-red-400/70 mt-0.5">{error}</p>
              </div>
            </div>
          )}
          <button
            onClick={onClose}
            className="w-full px-4 py-2 btn-secondary"
          >
            Close
          </button>
        </div>
      </div>
    </>
  );
}
