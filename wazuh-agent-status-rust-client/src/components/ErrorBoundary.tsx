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
            color: "#f43f5e",
            background: "#0b1120",
            height: "100vh",
            display: "flex",
            flexDirection: "column",
            gap: "12px",
            overflow: "auto",
          }}
        >
          <h2 style={{ color: "#f8fafc", margin: 0, fontSize: "16px" }}>
            ⚠️ Application Error
          </h2>
          <div
            style={{
              background: "rgba(244, 63, 94, 0.1)",
              border: "1px solid rgba(244, 63, 94, 0.2)",
              borderRadius: "8px",
              padding: "12px",
              color: "#f87171",
            }}
          >
            {this.state.error?.message ?? "Unknown error"}
          </div>
          {this.state.error?.stack && (
            <pre
              style={{
                background: "#111827",
                borderRadius: "8px",
                padding: "12px",
                color: "#94a3b8",
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
              window.location.reload();
            }}
            style={{
              background: "linear-gradient(135deg, #00aaff, #38bdf8)",
              border: "none",
              color: "white",
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
