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
          <div className="w-full max-w-md rounded-xl bg-bg-bolder border border-bd-danger p-6 flex flex-col gap-4">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-lg bg-bg-danger-faint flex items-center justify-center flex-shrink-0">
                <AlertTriangle size={16} className="text-fg-danger-moderate" />
              </div>
              <h1 className="text-body-sm font-medium text-fg-lighter">
                Something went wrong
              </h1>
            </div>

            <p className="text-body-sm text-fg-medium">
              The application encountered an unexpected error. This has been
              logged for investigation.
            </p>

            {this.state.error && (
              <details className="flex flex-col gap-2">
                <summary className="cursor-pointer text-caption text-fg-medium hover:text-fg-lighter">
                  Technical details
                </summary>
                <div className="rounded-lg bg-bg-strong border border-bd-faint p-3 text-caption font-mono text-fg-danger-moderate overflow-auto max-h-40">
                  <p className="font-semibold">{this.state.error.name}</p>
                  <p className="text-fg-medium">{this.state.error.message}</p>
                  {this.state.error.stack && (
                    <pre className="pt-2 text-fg-moderate">{this.state.error.stack}</pre>
                  )}
                </div>
              </details>
            )}

            <div className="flex gap-3">
              <Button
                variant="primary"
                size="lg"
                onClick={this.handleReset}
                className="flex-1"
              >
                Try again
              </Button>
              <Button
                variant="secondary"
                size="lg"
                onClick={this.handleReload}
                className="flex-1"
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
