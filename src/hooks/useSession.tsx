import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  BackupMode,
  BackupOptions,
  LogEntry,
  RestoreOptions,
  SessionDone,
  SessionProgress,
} from "../types/ipc";
import * as api from "../lib/api";
import {
  listenSessionDone,
  listenSessionError,
  listenSessionLog,
  listenSessionProgress,
} from "../lib/events";

const MAX_LOG_LINES = 500;

export type SessionState = "idle" | "running" | "done" | "error";

interface SessionContextValue {
  state: SessionState;
  mode: BackupMode | null;
  sessionId: string | null;
  progress: SessionProgress | null;
  logs: LogEntry[];
  summary: SessionDone["summary"] | null;
  error: string | null;
  startBackup: (options: BackupOptions) => Promise<void>;
  startRestore: (options: RestoreOptions) => Promise<void>;
  cancel: () => Promise<void>;
  reset: () => void;
}

const SessionContext = createContext<SessionContextValue | null>(null);

export function SessionProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<SessionState>("idle");
  const [mode, setMode] = useState<BackupMode | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [progress, setProgress] = useState<SessionProgress | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [summary, setSummary] = useState<SessionDone["summary"] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const stateRef = useRef(state);
  stateRef.current = state;

  const pushLog = useCallback((entry: LogEntry) => {
    setLogs((prev) => [...prev.slice(-(MAX_LOG_LINES - 1)), entry]);
  }, []);

  const reset = useCallback(() => {
    setState("idle");
    setMode(null);
    setSessionId(null);
    setProgress(null);
    setSummary(null);
    setError(null);
  }, []);

  useEffect(() => {
    const unProgress = listenSessionProgress((p) => setProgress(p));
    const unLog = listenSessionLog((entry) => pushLog(entry));
    const unDone = listenSessionDone((done: SessionDone) => {
      if (stateRef.current !== "running") return;
      setState("done");
      setSummary(done.summary);
      pushLog({ level: "info", ts: Date.now(), message: "Operasi selesai dengan sukses." });
    });
    const unError = listenSessionError((err) => {
      if (stateRef.current !== "running") return;
      setState("error");
      setError(err.error);
      pushLog({ level: "error", ts: Date.now(), message: err.error });
    });
    return () => {
      void unProgress.then((fn) => fn());
      void unLog.then((fn) => fn());
      void unDone.then((fn) => fn());
      void unError.then((fn) => fn());
    };
  }, [pushLog]);

  const startBackup = useCallback(
    async (options: BackupOptions) => {
      reset();
      setMode("backup");
      setState("running");
      try {
        const id = await api.startBackup(options);
        setSessionId(id);
      } catch (err) {
        setState("error");
        setError(String(err));
        pushLog({ level: "error", ts: Date.now(), message: String(err) });
      }
    },
    [pushLog, reset],
  );

  const startRestore = useCallback(
    async (options: RestoreOptions) => {
      reset();
      setMode("restore");
      setState("running");
      try {
        const id = await api.startRestore(options);
        setSessionId(id);
      } catch (err) {
        setState("error");
        setError(String(err));
        pushLog({ level: "error", ts: Date.now(), message: String(err) });
      }
    },
    [pushLog, reset],
  );

  const cancel = useCallback(async () => {
    try {
      if (sessionId) await api.cancelBackup(sessionId);
      pushLog({ level: "warn", ts: Date.now(), message: "Operasi dibatalkan. Checkpoint tersimpan untuk resume." });
    } catch (err) {
      pushLog({ level: "error", ts: Date.now(), message: String(err) });
    }
    setState("idle");
    setMode(null);
    setSessionId(null);
    setProgress(null);
    setSummary(null);
    setError(null);
  }, [sessionId, pushLog]);

  const value = useMemo(
    () => ({
      state,
      mode,
      sessionId,
      progress,
      logs,
      summary,
      error,
      startBackup,
      startRestore,
      cancel,
      reset,
    }),
    [state, mode, sessionId, progress, logs, summary, error, startBackup, startRestore, cancel, reset],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): SessionContextValue {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession harus dipakai di dalam SessionProvider");
  return ctx;
}
