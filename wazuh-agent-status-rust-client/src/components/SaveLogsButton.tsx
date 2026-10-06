
import type { SaveState } from "../hooks/useLogDownloader";

interface SaveLogsButtonProps {
  readonly downloadState: SaveState;
  readonly onSave: () => void;
  readonly title: string;
  readonly className?: string;
}

export function SaveLogsButton({
  downloadState,
  onSave,
  title,
  className = "",
}: SaveLogsButtonProps) {
  let iconAndText = (
    <>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>
      {" "}Save Logs
    </>
  );

  if (downloadState === "loading") {
    iconAndText = (
      <>
        <span className="logs-download-spinner" />
        {" "}Saving…
      </>
    );
  } else if (downloadState === "success") {
    iconAndText = (
      <>
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        {" "}Saved
      </>
    );
  }

  return (
    <button
      type="button"
      className={`logs-download-btn ${downloadState} ${className}`.trim()}
      onClick={onSave}
      disabled={downloadState === "loading"}
      title={title}
    >
      {iconAndText}
    </button>
  );
}
