import { useEffect, useRef } from "react";
import type { LogEntry } from "../../types/ipc";
import { formatTs } from "../../lib/format";
import { Icon } from "./Icon";

interface LogViewProps {
  entries: LogEntry[];
  emptyText?: string;
  maxHeight?: number;
}

function LevelIcon({ level }: { level: LogEntry["level"] }) {
  switch (level) {
    case "warn":
      return <Icon name="warning" size={14} className="log-row__icon log-row__icon--warn" />;
    case "error":
      return <Icon name="error" size={14} className="log-row__icon log-row__icon--error" />;
    default:
      return <Icon name="info" size={14} className="log-row__icon log-row__icon--info" />;
  }
}

export function LogView({ entries, emptyText = "Belum ada aktivitas.", maxHeight = 260 }: LogViewProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [entries.length]);

  return (
    <div className="logview" style={{ maxHeight }}>
      {entries.length === 0 ? (
        <div className="logview__empty">{emptyText}</div>
      ) : (
        <ul className="logview__list">
          {entries.map((entry, i) => (
            <li key={`${entry.ts}-${i}`} className={`log-row log-row--${entry.level}`}>
              <span className="log-row__time">{formatTs(entry.ts)}</span>
              <LevelIcon level={entry.level} />
              <span className="log-row__msg">{entry.message}</span>
            </li>
          ))}
          <div ref={bottomRef} />
        </ul>
      )}
    </div>
  );
}
