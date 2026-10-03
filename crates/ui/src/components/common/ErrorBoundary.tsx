import { Component, ReactNode } from "react";
import { AlertTriangle } from "lucide-react";
import { Button } from "../primitives/Button";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-full flex items-center justify-center p-6">
          <div className="w-full max-w-md rounded-xl bg-gray-750 border border-danger-700/50 p-6 flex flex-col gap-4">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-lg bg-danger-900/40 flex items-center justify-center flex-shrink-0">
                <AlertTriangle size={18} className="text-danger-400" />
              </div>
              <h1 className="text-base font-medium text-primary">
                Something went wrong
              </h1>
            </div>

            <p className="text-sm text-gray-300">
              The application encountered an unexpected error. This has been
              logged for investigation.
            </p>

            {this.state.error && (
              <details>
                <summary className="cursor-pointer text-xs text-gray-300 hover:text-primary">
                  Technical details
                </summary>
                <div className="mt-2 rounded-lg bg-gray-800 border border-gray-500/40 p-3 text-xs font-mono text-danger-300 overflow-auto max-h-40">
                  <p className="font-semibold">{this.state.error.name}</p>
                  <p className="text-gray-300">{this.state.error.message}</p>
                  {this.state.error.stack && (
                    <pre className="mt-2 text-gray-400">{this.state.error.stack}</pre>
                  )}
                </div>
              </details>
            )}

            <div className="flex gap-3">
              <Button
                variant="primary"
                size="none"
                onClick={this.handleReset}
                className="flex-1 py-2 text-sm"
              >
                Try again
              </Button>
              <Button
                variant="secondary"
                size="none"
                onClick={this.handleReload}
                className="flex-1 py-2 text-sm"
              >
                Reload app
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
