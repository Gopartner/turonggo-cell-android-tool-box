import type { SessionProgress } from "../../types/ipc";
import type { SessionState } from "../../hooks/useSession.tsx";
import { ProgressBar } from "./ProgressBar";
import { Button } from "./Button";
import { Icon } from "./Icon";

interface SessionPanelProps {
  state: SessionState;
  progress: SessionProgress | null;
  onCancel: () => void;
}

export function SessionPanel({ state, progress, onCancel }: SessionPanelProps) {
  return (
    <div className="session-panel">
      <div className="session-panel__head">
        <span className="session-panel__title">
          {state === "error" ? "Operasi gagal" : "Operasi sedang berjalan"}
        </span>
        <Button variant="danger" size="sm" icon={<Icon name="stop" size={15} />} onClick={onCancel}>
          Hentikan
        </Button>
      </div>
      {progress != null && (
        <ProgressBar
          percent={progress.percent}
          phase={progress.phase}
          currentPartition={progress.currentPartition}
          bytesRead={progress.bytesRead}
          bytesTotal={progress.bytesTotal}
        />
      )}
      {progress == null && <ProgressBar percent={0} phase="prepare" />}
      <p className="session-panel__hint">
        Jangan mencabut USB atau mematikan device selama operasi berlangsung.
      </p>
    </div>
  );
}
