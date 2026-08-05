// Pembungkus typed untuk semua listen()/emit.
// Komponen tidak pernah memanggil listen langsung — selalu lewat file ini.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DeviceStatus,
  DeviceSummary,
  LogEntry,
  SessionDone,
  SessionError,
  SessionProgress,
} from "../types/ipc";
import { composeDeviceStatus, isTauriRuntime } from "./api";
import { mockOn, mockOff } from "./mock";

export const CHANNELS = {
  deviceStatus: "device://status",
  sessionProgress: "session://progress",
  sessionLog: "session://log",
  sessionDone: "session://done",
  sessionError: "session://error",
} as const;

export type Unlisten = Promise<UnlistenFn>;

function mockListen<T>(channel: string, cb: (payload: T) => void): Unlisten {
  const handler = (payload: unknown) => cb(payload as T);
  mockOn(channel, handler);
  return Promise.resolve(() => mockOff(channel, handler));
}

// Wire payload device://status = DeviceSummary (docs/API.md §3);
// front-end mengomposisi DeviceStatus agar konsisten dengan getDeviceStatus().
export function listenDeviceStatus(cb: (status: DeviceStatus) => void): Unlisten {
  if (!isTauriRuntime()) return mockListen(CHANNELS.deviceStatus, cb);
  return listen<DeviceSummary>(CHANNELS.deviceStatus, (e) => cb(composeDeviceStatus(e.payload)));
}

export function listenSessionProgress(cb: (p: SessionProgress) => void): Unlisten {
  if (!isTauriRuntime()) return mockListen(CHANNELS.sessionProgress, cb);
  return listen<SessionProgress>(CHANNELS.sessionProgress, (e) => cb(e.payload));
}

export function listenSessionLog(cb: (entry: LogEntry) => void): Unlisten {
  if (!isTauriRuntime()) return mockListen(CHANNELS.sessionLog, cb);
  return listen<LogEntry>(CHANNELS.sessionLog, (e) => cb(e.payload));
}

export function listenSessionDone(cb: (done: SessionDone) => void): Unlisten {
  if (!isTauriRuntime()) return mockListen(CHANNELS.sessionDone, cb);
  return listen<SessionDone>(CHANNELS.sessionDone, (e) => cb(e.payload));
}

export function listenSessionError(cb: (err: SessionError) => void): Unlisten {
  if (!isTauriRuntime()) return mockListen(CHANNELS.sessionError, cb);
  return listen<SessionError>(CHANNELS.sessionError, (e) => cb(e.payload));
}
