// Pembungkus typed untuk semua pemanggilan invoke().
// Semua command mengikuti docs/API.md: balasan envelope ApiResponse<T>.
// Komponen TIDAK pernah memanggil invoke langsung — selalu lewat file ini.
import { invoke } from "@tauri-apps/api/core";
import type {
  AdbDeviceInfo,
  AdbRebootTarget,
  ApiResponse,
  AppSettings,
  BackupManifest,
  BackupOptions,
  BootloaderStatus,
  DeviceHealth,
  DeviceStatus,
  DeviceSummary,
  PartitionInfo,
  RestoreOptions,
  UsbPortInfo,
} from "../types/ipc";
import * as mock from "./mock";

export const isTauriRuntime = (): boolean => "__TAURI_INTERNALS__" in window;

export class ApiError extends Error {
  readonly code: string;
  readonly suggestion?: string;

  constructor(code: string, message: string, suggestion?: string) {
    super(message);
    this.name = "ApiError";
    this.code = code;
    this.suggestion = suggestion;
  }
}

// Envelope: sukses -> data; gagal -> lempar ApiError (code + suggestion).
function unwrap<T>(res: ApiResponse<T>): Promise<T> {
  if (!res.success) {
    return Promise.reject(
      new ApiError(res.error?.code ?? "INTERNAL", res.error?.message ?? "Terjadi kesalahan tidak dikenal", res.error?.suggestion),
    );
  }
  return Promise.resolve(res.data as T);
}

// Komposisi view-model DeviceStatus dari DeviceSummary (device_get_status)
// sesuai docs/API.md. Field Unisoc (chip/fdl/ports) diambil dari Device.extra.
export function composeDeviceStatus(summary: DeviceSummary): DeviceStatus {
  const primary = summary.devices[0] ?? null;
  const extra: Record<string, string> = Object.fromEntries(primary?.extra ?? []);
  const get = (key: string): string | undefined => extra[key];
  const bool = (value: string | undefined): boolean | undefined =>
    value === undefined ? undefined : value === "true";

  let ports: UsbPortInfo[] | undefined;
  try {
    if (get("ports")) ports = JSON.parse(get("ports")!) as UsbPortInfo[];
  } catch {
    ports = undefined;
  }

  return {
    mode: primary?.mode ?? "none",
    chip: get("chip"),
    chipId: get("chipId"),
    fdlLoaded: bool(get("fdlLoaded")),
    transport: get("transport"),
    manufacturer: get("manufacturer"),
    model: get("model"),
    productName: get("productName"),
    codename: get("codename"),
    androidVersion: get("androidVersion"),
    buildFingerprint: get("buildFingerprint"),
    securityPatch: get("securityPatch"),
    bootloaderStatus: get("bootloaderStatus") as BootloaderStatus | undefined,
    slotActive: get("slotActive"),
    isAbDevice: bool(get("isAbDevice")),
    driverOk: bool(get("driverOk")),
    ports,
  };
}

// —— Device Detection ——

export function getDeviceStatus(): Promise<DeviceStatus> {
  const res: Promise<ApiResponse<DeviceSummary>> = isTauriRuntime()
    ? invoke<ApiResponse<DeviceSummary>>("device_get_status")
    : Promise.resolve(mock.mockDeviceSummary());
  return res.then(unwrap).then(composeDeviceStatus);
}

export function getPorts(): Promise<UsbPortInfo[]> {
  const res: Promise<ApiResponse<UsbPortInfo[]>> = isTauriRuntime()
    ? invoke<ApiResponse<UsbPortInfo[]>>("device_get_ports")
    : Promise.resolve(mock.mockGetPorts());
  return res.then(unwrap);
}

export function getHealth(): Promise<DeviceHealth> {
  const res: Promise<ApiResponse<DeviceHealth>> = isTauriRuntime()
    ? invoke<ApiResponse<DeviceHealth>>("device_get_health")
    : Promise.resolve(mock.mockGetHealth());
  return res.then(unwrap);
}

// —— ADB & Fastboot ——

export function adbReboot(deviceId: string, target: AdbRebootTarget): Promise<void> {
  const res: Promise<ApiResponse<null>> = isTauriRuntime()
    ? invoke<ApiResponse<null>>("device_adb_reboot", { deviceId, target })
    : Promise.resolve(mock.mockAdbReboot(deviceId, target));
  return res.then(unwrap).then(() => undefined);
}

export function adbShell(deviceId: string, command: string): Promise<string> {
  const res: Promise<ApiResponse<string>> = isTauriRuntime()
    ? invoke<ApiResponse<string>>("device_adb_shell", { deviceId, command })
    : Promise.resolve(mock.mockAdbShell(deviceId, command));
  return res.then(unwrap);
}

export function adbInfo(deviceId: string): Promise<AdbDeviceInfo> {
  const res: Promise<ApiResponse<AdbDeviceInfo>> = isTauriRuntime()
    ? invoke<ApiResponse<AdbDeviceInfo>>("device_adb_info", { deviceId })
    : Promise.resolve(mock.mockAdbDeviceInfo(deviceId));
  return res.then(unwrap);
}

export function fastbootGetVar(deviceId: string, varName: string): Promise<Record<string, string>> {
  const res: Promise<ApiResponse<Record<string, string>>> = isTauriRuntime()
    ? invoke<ApiResponse<Record<string, string>>>("device_fastboot_getvar", { deviceId, var: varName })
    : Promise.resolve(mock.mockFastbootGetVar(deviceId, varName));
  return res.then(unwrap);
}

export function fastbootFlash(
  deviceId: string,
  partitionName: string,
  filePath: string,
  verifyChecksum?: boolean,
): Promise<{ bytesWritten: number; timeElapsedMs: number }> {
  const res: Promise<ApiResponse<{ bytesWritten: number; timeElapsedMs: number }>> = isTauriRuntime()
    ? invoke<ApiResponse<{ bytesWritten: number; timeElapsedMs: number }>>("device_fastboot_flash", {
        deviceId,
        partitionName,
        filePath,
        verifyChecksum,
      })
    : Promise.resolve(mock.mockFastbootFlash(deviceId, partitionName, filePath, verifyChecksum));
  return res.then(unwrap);
}

// —— Partition & Backup/Restore ——

export function partitionList(): Promise<PartitionInfo[]> {
  const res: Promise<ApiResponse<PartitionInfo[]>> = isTauriRuntime()
    ? invoke<ApiResponse<PartitionInfo[]>>("partition_list")
    : Promise.resolve(mock.mockPartitionList());
  return res.then(unwrap);
}

export function startBackup(options: BackupOptions): Promise<string> {
  const res: Promise<ApiResponse<{ sessionId: string }>> = isTauriRuntime()
    ? invoke<ApiResponse<{ sessionId: string }>>("backup_start", { options })
    : Promise.resolve(mock.mockStartBackup(options));
  return res.then(unwrap).then((d) => d.sessionId);
}

export function cancelBackup(sessionId: string): Promise<void> {
  const res: Promise<ApiResponse<null>> = isTauriRuntime()
    ? invoke<ApiResponse<null>>("backup_cancel", { sessionId })
    : Promise.resolve(mock.mockCancelBackup(sessionId));
  return res.then(unwrap).then(() => undefined);
}

export function startRestore(options: RestoreOptions): Promise<string> {
  const res: Promise<ApiResponse<{ sessionId: string }>> = isTauriRuntime()
    ? invoke<ApiResponse<{ sessionId: string }>>("restore_start", { options })
    : Promise.resolve(mock.mockStartRestore(options));
  return res.then(unwrap).then((d) => d.sessionId);
}

// —— Settings & Lain-lain ——

export function getSettings(): Promise<AppSettings> {
  const res: Promise<ApiResponse<AppSettings>> = isTauriRuntime()
    ? invoke<ApiResponse<AppSettings>>("settings_get")
    : Promise.resolve(mock.mockGetSettings());
  return res.then(unwrap);
}

export function setSettings(settings: AppSettings): Promise<void> {
  const res: Promise<ApiResponse<null>> = isTauriRuntime()
    ? invoke<ApiResponse<null>>("settings_set", { settings })
    : Promise.resolve(mock.mockSetSettings(settings));
  return res.then(unwrap).then(() => undefined);
}

export function appVersion(): Promise<string> {
  const res: Promise<ApiResponse<string>> = isTauriRuntime()
    ? invoke<ApiResponse<string>>("app_version")
    : Promise.resolve(mock.mockAppVersion());
  return res.then(unwrap);
}

export function pickFolder(): Promise<string | null> {
  const res: Promise<ApiResponse<string | null>> = isTauriRuntime()
    ? invoke<ApiResponse<string | null>>("dialog_pick_folder")
    : Promise.resolve(mock.mockPickFolder());
  return res.then(unwrap);
}

export function readBackupFolder(dir: string): Promise<BackupManifest> {
  const res: Promise<ApiResponse<BackupManifest>> = isTauriRuntime()
    ? invoke<ApiResponse<BackupManifest>>("backup_read_manifest", { dir })
    : Promise.resolve(mock.mockReadBackupFolder(dir));
  return res.then(unwrap);
}
