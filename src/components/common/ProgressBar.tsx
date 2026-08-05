import { formatBytes } from "../../lib/format";
import type { SessionPhase } from "../../types/ipc";

interface ProgressBarProps {
  percent: number;
  phase?: SessionPhase;
  currentPartition?: string;
  bytesRead?: number;
  bytesTotal?: number;
  size?: "sm" | "lg";
}

const PHASE_LABEL: Record<SessionPhase, string> = {
  prepare: "Menyiapkan",
  read: "Membaca",
  write: "Menulis",
  verify: "Memverifikasi",
  finalize: "Menyelesaikan",
};

export function ProgressBar({
  percent,
  phase,
  currentPartition,
  bytesRead,
  bytesTotal,
  size = "lg",
}: ProgressBarProps) {
  const clamped = Math.max(0, Math.min(100, percent));
  return (
    <div className={`progress${size === "sm" ? " progress--sm" : ""}`}>
      <div className="progress__top">
        <span className="progress__label">
          {phase != null && <em className="progress__phase">{PHASE_LABEL[phase]}</em>}
          {currentPartition != null && <span className="progress__part">{currentPartition}</span>}
        </span>
        <span className="progress__value">{clamped.toFixed(clamped % 1 === 0 ? 0 : 1)}%</span>
      </div>
      <div className="progress__track" role="progressbar" aria-valuenow={clamped} aria-valuemin={0} aria-valuemax={100}>
        <div className="progress__fill" style={{ width: `${clamped}%` }} />
      </div>
      {bytesRead != null && bytesTotal != null && bytesTotal > 0 && (
        <div className="progress__sub">
          <span>
            {formatBytes(bytesRead)} / {formatBytes(bytesTotal)}
          </span>
        </div>
      )}
    </div>
  );
}
