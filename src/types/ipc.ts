// Kontrak IPC front-end <-> back-end.
// SUMBER TUNGGAL: docs/API.md. Tipe di sini disalin dari docs/API.md
// (jangan ubah field tanpa mengubah docs/API.md lebih dulu).

// Set nama mode final (disepakati dengan back-end, lihat REQUESTS.md REQ-001).
// MVP: none | adb | fastboot | recovery | unisoc_download.
// Cadangan chipset lain (PRD §5.4-5.7): mtk_brom | mtk_preloader | qcom_edl | samsung_download.
export type DeviceMode =
  | "none"
  | "adb"
  | "fastboot"
  | "recovery"
  | "unisoc_download"
  | "mtk_brom"
  | "mtk_preloader"
  | "qcom_edl"
  | "samsung_download";

export type AdbRebootTarget =
  | "system"
  | "recovery"
  | "bootloader"
  | "download"
  | "edl";

export type BootloaderStatus = "locked" | "unlocked" | "unknown";

// —— Envelope respon standar (dipakai SEMUA command IPC) ——
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    suggestion?: string;
  };
}

export interface UsbPortInfo {
  name: string;
  state: "connected" | "error" | "idle";
  detail?: string;
}

// —— Satu perangkat hasil scan ——
export interface Device {
  mode: DeviceMode;
  serial: string;
  state: string; // adb: "device"|"offline"|"unauthorized"; fastboot: "fastboot"|"fastbootd"
  extra?: [string, string][];
}

// —— Hasil scan: payload device_get_status & event device://status ——
export interface DeviceSummary {
  connected: boolean;
  devices: Device[];
  lastScanMs: number;
  warnings?: string[];
}

// —— Info perangkat dari getprop (adapter ADB) ——
export interface AdbDeviceInfo {
  serial: string;
  manufacturer?: string;
  model?: string;
  productName?: string;
  codename?: string;
  brand?: string;
  androidVersion?: string;
  buildFingerprint?: string;
  securityPatch?: string;
  sdk?: string;
}

// —— Health score device (REQ-003) ——
export interface DeviceHealth {
  score: number;
  riskLevel: "low" | "medium" | "high";
  issues: { severity: "info" | "warn" | "critical"; message: string }[];
  recommendedAction?: string;
  checkedAt: number;
}

export interface DeviceStatus {
  mode: DeviceMode;
  chip?: string;
  chipId?: string;
  fdlLoaded?: boolean;
  transport?: string;
  manufacturer?: string;
  model?: string;
  productName?: string;
  codename?: string;
  androidVersion?: string;
  buildFingerprint?: string;
  securityPatch?: string;
  bootloaderStatus?: BootloaderStatus;
  slotActive?: string;
  isAbDevice?: boolean;
  driverOk?: boolean;
  ports?: UsbPortInfo[];
}

export interface PartitionInfo {
  name: string;
  size: number;
  offset?: number;
  isUserPartition?: boolean;
}

export interface BackupOptions {
  sessionId?: string;
  partitions: string[];
  targetDir: string;
  resume?: boolean;
}

export interface RestoreOptions {
  partitions: { name: string; file: string }[];
}

export type SessionPhase = "prepare" | "read" | "write" | "verify" | "finalize";

export interface SessionProgress {
  sessionId: string;
  phase: SessionPhase;
  percent: number;
  bytesRead: number;
  bytesTotal: number;
  currentPartition?: string;
}

export type LogLevel = "info" | "warn" | "error";

export interface LogEntry {
  level: LogLevel;
  ts: number;
  message: string;
}

export interface SessionSummary {
  partitionsDone: number;
  bytesTotal: number;
  durationMs: number;
  targetDir?: string;
}

export interface SessionDone {
  sessionId: string;
  summary: SessionSummary;
}

export interface SessionError {
  sessionId: string;
  error: string;
}

export interface AppSettings {
  chipset: string;
  workDir: string;
  resumeEnabled: boolean;
  allowRestoreToStock: boolean;
}

export interface BackupManifestPartition {
  name: string;
  size: number;
  file: string;
  sha256?: string;
}

export interface BackupManifest {
  chipset: string;
  createdAt?: number;
  partitions: BackupManifestPartition[];
}

export type BackupMode = "backup" | "restore";
