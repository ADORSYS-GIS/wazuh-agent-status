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
        <div
          style={{
            padding: "24px",
            fontFamily: "monospace",
            fontSize: "13px",
            color: "var(--error)",
            background: "var(--bg)",
            height: "100vh",
            display: "flex",
            flexDirection: "column",
            gap: "12px",
            overflow: "auto",
          }}
        >
          <h2 style={{ color: "var(--text)", margin: 0, fontSize: "16px" }}>
            ⚠️ Application Error
          </h2>
          <div
            style={{
              background: "var(--overlay-white-05)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
              padding: "12px",
              color: "var(--error)",
            }}
          >
            {this.state.error?.message ?? "Unknown error"}
          </div>
          {this.state.error?.stack && (
            <pre
              style={{
                background: "var(--sidebar-bg)",
                borderRadius: "8px",
                padding: "12px",
                color: "var(--text-dim)",
                fontSize: "11px",
                lineHeight: 1.6,
                overflow: "auto",
                maxHeight: "400px",
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
              }}
            >
              {this.state.error.stack}
            </pre>
          )}
          <button
            onClick={() => {
              this.setState({ hasError: false, error: null });
              globalThis.location.reload();
            }}
            style={{
              background: "var(--primary-metallic)",
              border: "none",
              color: "var(--text-on-primary)",
              padding: "10px 20px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: 600,
              cursor: "pointer",
              marginTop: "auto",
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
