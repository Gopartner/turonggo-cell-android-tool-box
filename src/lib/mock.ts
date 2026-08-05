// Simulator backend untuk preview UI di browser (npm run dev tanpa Tauri).
// Dipakai OTOMATIS hanya bila app tidak berjalan di dalam runtime Tauri.
// Bentuk JSON meniru docs/API.md (envelope ApiResponse<T> + contoh payload).
// Saat command Rust sudah diimplementasikan, bagian ini tidak terpakai.
import type {
  AdbDeviceInfo,
  AdbRebootTarget,
  ApiResponse,
  AppSettings,
  BackupManifest,
  BackupOptions,
  DeviceHealth,
  DeviceSummary,
  LogEntry,
  PartitionInfo,
  RestoreOptions,
  SessionDone,
  SessionProgress,
  UsbPortInfo,
} from "../types/ipc";
import { formatBytes } from "./format";

export const MOCK_SETTINGS_KEY = "spd.mock.settings";
export const MOCK_TARGET_DIR = "D:\\spd_backup_tool\\images";

const OK = <T>(data: T): ApiResponse<T> => ({ success: true, data });
const ERR = <T>(code: string, message: string, suggestion?: string): ApiResponse<T> => ({
  success: false,
  error: { code, message, suggestion },
});

const PARTITIONS: PartitionInfo[] = [
  { name: "splloader", size: 1 * 1024 * 1024, isUserPartition: false },
  { name: "bootloader", size: 5 * 1024 * 1024, isUserPartition: false },
  { name: "vbmeta", size: 2 * 1024 * 1024, isUserPartition: false },
  { name: "vbmeta_system", size: 2 * 1024 * 1024, isUserPartition: false },
  { name: "boot", size: 32 * 1024 * 1024, isUserPartition: false },
  { name: "dtbo", size: 16 * 1024 * 1024, isUserPartition: false },
  { name: "dtb_a", size: 2 * 1024 * 1024, isUserPartition: false },
  { name: "hypervsior_a", size: 8 * 1024 * 1024, isUserPartition: false },
  { name: "recovery", size: 32 * 1024 * 1024, isUserPartition: false },
  { name: "l_modem", size: 128 * 1024 * 1024, isUserPartition: false },
  { name: "l_gsm", size: 16 * 1024 * 1024, isUserPartition: false },
  { name: "l_dsp", size: 16 * 1024 * 1024, isUserPartition: false },
  { name: "l_ldsp", size: 8 * 1024 * 1024, isUserPartition: false },
  { name: "l_gdsp", size: 4 * 1024 * 1024, isUserPartition: false },
  { name: "metadata", size: 16 * 1024 * 1024, isUserPartition: false },
  { name: "persist", size: 16 * 1024 * 1024, isUserPartition: false },
  { name: "miscdata", size: 1 * 1024 * 1024, isUserPartition: false },
  { name: "super", size: 8 * 1024 * 1024 * 1024, isUserPartition: false },
  { name: "nv", size: 1 * 1024 * 1024, isUserPartition: true },
  { name: "prodnv", size: 64 * 1024 * 1024, isUserPartition: true },
  { name: "l_fixnv1", size: 8 * 1024 * 1024, isUserPartition: true },
  { name: "l_fixnv2", size: 8 * 1024 * 1024, isUserPartition: true },
  { name: "l_runtimenv", size: 2 * 1024 * 1024, isUserPartition: true },
  { name: "calinv", size: 4 * 1024 * 1024, isUserPartition: true },
];

type MockListener = (payload: unknown) => void;

const listeners = new Map<string, MockListener[]>();
const activeTicks = new Set<ReturnType<typeof setInterval>>();

export function mockOn(channel: string, cb: MockListener): void {
  const list = listeners.get(channel) ?? [];
  list.push(cb);
  listeners.set(channel, list);
}

export function mockOff(channel: string, cb: MockListener): void {
  const list = listeners.get(channel) ?? [];
  listeners.set(
    channel,
    list.filter((fn) => fn !== cb),
  );
}

function emit(channel: string, payload: unknown): void {
  for (const cb of listeners.get(channel) ?? []) cb(payload);
}

function log(level: LogEntry["level"], message: string): void {
  emit("session://log", { level, ts: Date.now(), message } satisfies LogEntry);
}

// —— Device Detection ——

export function mockDeviceSummary(): ApiResponse<DeviceSummary> {
  return OK({
    connected: true,
    devices: [
      {
        mode: "unisoc_download",
        serial: "U2S-00130000",
        state: "spd-download",
        extra: [
          ["chip", "ums9230"],
          ["chipId", "0x00130000"],
          ["fdlLoaded", "true"],
          ["transport", "libusb"],
          ["manufacturer", "Realme"],
          ["model", "RMX3760"],
          ["productName", "realme C53"],
          ["codename", "RMX3760"],
          ["androidVersion", "14 (Upside Down Cake)"],
          ["buildFingerprint", "realme/RMX3760/RMX3760:14/UP1A.231005.007/S.175:user/release-keys"],
          ["securityPatch", "2025-03-05"],
          ["bootloaderStatus", "unlocked"],
          ["slotActive", "a"],
          ["isAbDevice", "true"],
          [
            "ports",
            JSON.stringify([
              { name: "SPRD U2S Diag (COM3)", state: "connected", detail: "SPD Download Mode" },
              { name: "USB Debugging (ADB)", state: "idle", detail: "adb transport tidak aktif di mode Download" },
            ] satisfies UsbPortInfo[]),
          ],
          ["driverOk", "true"],
        ],
      },
    ],
    lastScanMs: 42,
    warnings: [],
  });
}

export function mockGetPorts(): ApiResponse<UsbPortInfo[]> {
  return OK([
    { name: "SPRD U2S Diag (COM3)", state: "connected", detail: "SPD Download Mode" },
    { name: "USB Debugging (ADB)", state: "idle", detail: "adb transport tidak aktif di mode Download" },
  ]);
}

export function mockGetHealth(): ApiResponse<DeviceHealth> {
  return OK({
    score: 85,
    riskLevel: "medium",
    issues: [
      { severity: "warn", message: "Bootloader dalam status unlocked (risiko keamanan medium)." },
    ],
    recommendedAction: "Aktifkan kembali pengunci bootloader bila perangkat tidak dipakai untuk development.",
    checkedAt: Date.now(),
  });
}

// —— ADB & Fastboot ——

export function mockAdbReboot(_deviceId: string, target: AdbRebootTarget): ApiResponse<null> {
  log("warn", `Mode preview: kirim reboot ke target "${target}" (backend belum terhubung).`);
  return OK(null);
}

export function mockAdbShell(_deviceId: string, _command: string): ApiResponse<string> {
  return OK("console=ttyS1,115200n8 root=/dev/...\n");
}

export function mockAdbDeviceInfo(deviceId: string): ApiResponse<AdbDeviceInfo> {
  return OK({
    serial: deviceId,
    manufacturer: "Realme",
    model: "RMX3760",
    productName: "RMX3760",
    codename: "RE5D4L",
    brand: "realme",
    androidVersion: "13",
    buildFingerprint: "realme/RMX3760/RMX3760:13/RKQ1.211103.002/S.175:user/release-keys",
    securityPatch: "2024-01-05",
    sdk: "33",
  });
}

export function mockFastbootGetVar(_deviceId: string, _varName: string): ApiResponse<Record<string, string>> {
  return OK({
    version: "0.4",
    product: "RMX3760",
    "slot-count": "2",
    "current-slot": "a",
    "partition-type:boot": "raw",
  });
}

export function mockFastbootFlash(
  _deviceId: string,
  partitionName: string,
  _filePath: string,
  _verifyChecksum?: boolean,
): ApiResponse<{ bytesWritten: number; timeElapsedMs: number }> {
  log("info", `Mode preview: flash partisi ${partitionName} (backend belum terhubung).`);
  return OK({ bytesWritten: 67108864, timeElapsedMs: 4210 });
}

// —— Partition & Backup/Restore ——

export function mockPartitionList(): ApiResponse<PartitionInfo[]> {
  return OK(PARTITIONS.map((p) => ({ ...p })));
}

export function mockStartBackup(options: BackupOptions): ApiResponse<{ sessionId: string }> {
  const sessionId = `backup-${Date.now().toString(36)}`;
  const total = PARTITIONS.filter((p) => options.partitions.includes(p.name)).reduce(
    (sum, p) => sum + p.size,
    0,
  );

  emit("session://log", { level: "info", ts: Date.now(), message: "Mode preview: simulasi backup berjalan (backend belum terhubung)." });

  let emitted = 0;
  const tick = setInterval(() => {
    emitted += 1;
    const step = Math.min(emitted / 60, 1);
    const idx = Math.min(PARTITIONS.length - 1, Math.floor(emitted / 3));
    const part = PARTITIONS[idx];

    emit("session://progress", {
      sessionId,
      phase: "read",
      currentPartition: part.name,
      bytesRead: Math.floor(total * step),
      bytesTotal: total,
      percent: Math.round(step * 100),
    } satisfies SessionProgress);

    if (emitted % 3 === 0) {
      log("info", `Selesai membaca ${part.name} (${formatBytes(part.size)}).`);
    }

    if (step >= 1) {
      clearInterval(tick);
      activeTicks.delete(tick);
      log("info", "Backup selesai.");
      emit("session://done", {
        sessionId,
        summary: {
          partitionsDone: PARTITIONS.filter((p) => options.partitions.includes(p.name)).length,
          bytesTotal: total,
          durationMs: 60000,
          targetDir: options.targetDir,
        },
      } satisfies SessionDone);
    }
  }, 600);

  activeTicks.add(tick);
  return OK({ sessionId });
}

export function mockCancelBackup(_sessionId: string): ApiResponse<null> {
  for (const tick of activeTicks) {
    clearInterval(tick);
    activeTicks.delete(tick);
  }
  log("warn", "Operasi dihentikan pada posisi aman terakhir.");
  return OK(null);
}

export function mockStartRestore(_options: RestoreOptions): ApiResponse<{ sessionId: string }> {
  const sessionId = `restore-${Date.now().toString(36)}`;
  let emitted = 0;
  const tick = setInterval(() => {
    emitted += 1;
    const step = Math.min(emitted / 40, 1);
    emit("session://progress", {
      sessionId,
      phase: "write",
      currentPartition: PARTITIONS[Math.floor(emitted / 2)].name,
      bytesRead: 0,
      bytesTotal: 100,
      percent: Math.round(step * 100),
    } satisfies SessionProgress);
    if (emitted % 4 === 0) log("info", "Menulis blok ke partisi ...");
    if (step >= 1) {
      clearInterval(tick);
      activeTicks.delete(tick);
      log("info", "Restore selesai. Cabut USB lalu verifikasi boot.");
      emit("session://done", {
        sessionId,
        summary: { partitionsDone: 16, bytesTotal: 0, durationMs: 40000 },
      } satisfies SessionDone);
    }
  }, 500);
  activeTicks.add(tick);
  return OK({ sessionId });
}

// —— Settings & Lain-lain ——

export function mockGetSettings(): ApiResponse<AppSettings> {
  try {
    const raw = localStorage.getItem(MOCK_SETTINGS_KEY);
    if (raw) return OK(JSON.parse(raw) as AppSettings);
  } catch {
    // fallback ke default di bawah
  }
  return OK({
    chipset: "ums9230",
    workDir: MOCK_TARGET_DIR,
    resumeEnabled: true,
    allowRestoreToStock: false,
  });
}

export function mockSetSettings(settings: AppSettings): ApiResponse<null> {
  localStorage.setItem(MOCK_SETTINGS_KEY, JSON.stringify(settings));
  return OK(null);
}

export function mockAppVersion(): ApiResponse<string> {
  return OK("0.1.0");
}

export function mockPickFolder(): ApiResponse<string | null> {
  return OK(MOCK_TARGET_DIR);
}

export function mockReadBackupFolder(dir: string): ApiResponse<BackupManifest> {
  return OK({
    chipset: "ums9230",
    createdAt: Date.now(),
    partitions: PARTITIONS.filter((p) => !p.name.startsWith("l_")).map((p) => ({
      name: p.name,
      size: p.size,
      file: `${dir}\\${p.name}.img`,
      sha256: "a".repeat(64),
    })),
  });
}

// Simulasi error (dipakai untuk menguji UI saat command gagal).
export function mockError(): ApiResponse<never> {
  return ERR("ERR_DRIVER_MISSING", "Driver USB SPD Download tidak ditemukan.", "Instal driver via Settings > Install Drivers.");
}
