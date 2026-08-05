import type { DeviceMode } from "../../types/ipc";

const MODE_META: Record<DeviceMode, { label: string; className: string }> = {
  none: { label: "Belum Terdeteksi", className: "mode--none" },
  adb: { label: "ADB", className: "mode--adb" },
  fastboot: { label: "Fastboot", className: "mode--fastboot" },
  recovery: { label: "Recovery", className: "mode--recovery" },
  unisoc_download: { label: "SPD Download", className: "mode--download" },
  mtk_brom: { label: "MTK BROM", className: "mode--mtk" },
  mtk_preloader: { label: "MTK Preloader", className: "mode--mtk" },
  qcom_edl: { label: "Qualcomm EDL", className: "mode--qcom" },
  samsung_download: { label: "Samsung Download", className: "mode--samsung" },
};

interface ModeBadgeProps {
  mode: DeviceMode;
}

export function ModeBadge({ mode }: ModeBadgeProps) {
  const meta = MODE_META[mode];
  return (
    <span className={`mode-badge ${meta.className}`}>
      <span className="mode-badge__dot" />
      {meta.label}
    </span>
  );
}
