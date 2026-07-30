import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[ErrorBoundary] Caught an error:", error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <h2>⚠️ Application Error</h2>
          <div className="error-boundary-card">
            {this.state.error?.message ?? "Unknown error"}
          </div>
          {this.state.error?.stack && (
            <pre className="error-boundary-stack">
              {this.state.error.stack}
            </pre>
          )}
          <button
            type="button"
            className="error-boundary-reload"
            onClick={() => {
              this.setState({ hasError: false, error: null });
              globalThis.location.reload();
            }}
          >
            Reload
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
