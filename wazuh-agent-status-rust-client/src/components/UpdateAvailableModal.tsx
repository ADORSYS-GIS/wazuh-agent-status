interface UpdateAvailableModalProps {
  currentVersion: string;
  latestVersion: string;
  onOpenUpdates: () => void;
  onRemindLater: () => void;
}

export function UpdateAvailableModal({
  currentVersion,
  latestVersion,
  onOpenUpdates,
  onRemindLater,
}: Readonly<UpdateAvailableModalProps>) {
  return (
    <div className="update-modal-backdrop">
      <div className="update-modal">
        <div className="update-modal-header">
          <div className="update-modal-title">
            <span className="update-status-badge available">AVAILABLE</span>
            <span>New version detected</span>
          </div>
        </div>

        <div className="update-modal-body">
          <div className="update-available-copy">
            <p>
              There is a newer version of Wazuh Agent Status than the one installed locally.
            </p>
            <p>
              Local: <strong>v{currentVersion}</strong> · Latest: <strong>v{latestVersion}</strong>
            </p>
            <p>
              Open the Updates tab to review the release and start the update when you are ready.
            </p>
          </div>

          <div className="update-available-actions">
            <button type="button" className="update-available-button primary" onClick={onOpenUpdates}>
              Open Updates
            </button>
            <button type="button" className="update-available-button secondary" onClick={onRemindLater}>
              Remind Me Later
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}