import { useDevice } from "../../hooks/useDevice";
import type { DeviceStatus } from "../../types/ipc";
import { Card } from "../common/Card";
import { Icon } from "../common/Icon";
import { ModeBadge } from "./ModeBadge";

function InfoRow({ label, value, mono = false }: { label: string; value?: string; mono?: boolean }) {
  return (
    <div className="info-row">
      <span className="info-row__label">{label}</span>
      <span className={`info-row__value${mono ? " mono" : ""}`}>{value ?? "-"}</span>
    </div>
  );
}

function formatChipId(status: DeviceStatus): string | undefined {
  return status.chipId != null ? status.chipId : undefined;
}

export function DeviceInfo() {
  const { status } = useDevice();

  return (
    <div className="device-info">
      <Card
        title="Device Summary"
        subtitle="Ringkasan perangkat yang terdeteksi"
        actions={<ModeBadge mode={status.mode} />}
      >
        <div className="info-grid">
          <InfoRow label="Mode koneksi" value={status.mode.toUpperCase()} mono />
          <InfoRow label="Chipset" value={status.chip} mono />
          <InfoRow label="Chip ID" value={formatChipId(status)} mono />
          <InfoRow label="FDL dimuat" value={status.fdlLoaded === true ? "Ya" : "Tidak"} />
          <InfoRow label="Transport" value={status.transport} mono />
          <InfoRow label="Manufacturer" value={status.manufacturer} />
          <InfoRow label="Model" value={status.model} />
          <InfoRow label="Product name" value={status.productName} />
          <InfoRow label="Codename" value={status.codename} />
          <InfoRow label="Android version" value={status.androidVersion} />
          <InfoRow label="Security patch" value={status.securityPatch} />
          <InfoRow label="Build fingerprint" value={status.buildFingerprint} mono />
          <InfoRow
            label="Bootloader"
            value={
              status.bootloaderStatus == null
                ? undefined
                : status.bootloaderStatus === "locked"
                  ? "Locked"
                  : "Unlocked"
            }
          />
          <InfoRow label="Slot aktif" value={status.slotActive} mono />
          <InfoRow label="A/B partition" value={status.isAbDevice === true ? "Ya" : "Tidak"} />
        </div>
      </Card>

      <Card
        title="Driver & Port USB"
        subtitle="Diagnostik koneksi dan status driver"
        actions={
          <span className={`flag${status.driverOk === true ? " flag--on" : ""}`}>
            <Icon name="harddrive" size={14} />
            {status.driverOk === true ? "Driver OK" : "Driver bermasalah"}
          </span>
        }
      >
        <ul className="port-list">
          {(status.ports ?? []).length === 0 ? (
            <li className="port-list__empty">Tidak ada port terdeteksi.</li>
          ) : (
            status.ports!.map((port) => (
              <li key={port.name} className={`port-row port-row--${port.state}`}>
                <span className="port-row__dot" />
                <div className="port-row__main">
                  <span className="port-row__name">{port.name}</span>
                  {port.detail != null && <span className="port-row__detail">{port.detail}</span>}
                </div>
                <span className="port-row__state">
                  {port.state === "connected" ? "Terhubung" : port.state === "error" ? "Error" : "Idle"}
                </span>
              </li>
            ))
          )}
        </ul>
      </Card>
    </div>
  );
}
