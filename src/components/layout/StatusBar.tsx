import { useDevice } from "../../hooks/useDevice.tsx";
import { useSession } from "../../hooks/useSession.tsx";
import { formatPercent } from "../../lib/format";
import { Icon } from "../common/Icon";
import { ModeBadge } from "../device/ModeBadge";

export function StatusBar() {
  const { status } = useDevice();
  const { state, progress, sessionId } = useSession();

  return (
    <footer className="statusbar">
      <div className="statusbar__group">
        <ModeBadge mode={status.mode} />
        {status.chip != null && <span className="statusbar__chip">{status.chip}</span>}
        {status.fdlLoaded === true && (
          <span className="statusbar__tag">
            <Icon name="chip" size={13} />
            FDL
          </span>
        )}
        {status.transport != null && <span className="statusbar__tag">{status.transport}</span>}
      </div>

      <div className="statusbar__session">
        {state === "running" && progress != null ? (
          <>
            <span className="statusbar__spinner" />
            <span className="statusbar__text">
              {sessionId ?? "session"} · {formatPercent(progress.percent)}
            </span>
          </>
        ) : state === "done" ? (
          <span className="statusbar__text statusbar__text--ok">Selesai</span>
        ) : state === "error" ? (
          <span className="statusbar__text statusbar__text--err">Gagal</span>
        ) : (
          <span className="statusbar__text statusbar__text--muted">Idle</span>
        )}
      </div>
    </footer>
  );
}
