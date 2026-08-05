import { useDevice } from "../../hooks/useDevice.tsx";
import { Card } from "../common/Card";
import { Icon } from "../common/Icon";
import { Button } from "../common/Button";
import { ModeBadge } from "./ModeBadge";

export function DeviceCard() {
  const { status, isLoading, scan } = useDevice();

  return (
    <Card
      title="Perangkat"
      subtitle="Status deteksi device melalui USB"
      actions={
        <Button
          variant="secondary"
          size="sm"
          icon={<Icon name="scan" size={15} />}
          onClick={() => void scan()}
          disabled={isLoading}
        >
          Pindai
        </Button>
      }
    >
      <div className="device-card">
        <div className="device-card__icon">
          <Icon name="phone" size={26} />
        </div>
        <div className="device-card__main">
          <ModeBadge mode={status.mode} />
          <div className="device-card__row">
            <span className="device-card__label">Perangkat</span>
            <span className="device-card__value">
              {status.model != null ? `${status.manufacturer ?? ""} ${status.model}`.trim() : "-"}
            </span>
          </div>
          <div className="device-card__row">
            <span className="device-card__label">Chipset</span>
            <span className="device-card__value">{status.chip ?? "-"}</span>
          </div>
          <div className="device-card__row">
            <span className="device-card__label">Android</span>
            <span className="device-card__value">{status.androidVersion ?? "-"}</span>
          </div>
          <div className="device-card__row">
            <span className="device-card__label">Transport</span>
            <span className="device-card__value">{status.transport ?? "-"}</span>
          </div>
        </div>
        <div className="device-card__flags">
          <span className={`flag${status.fdlLoaded === true ? " flag--on" : ""}`}>
            <Icon name="chip" size={14} />
            FDL
          </span>
          {status.bootloaderStatus != null && (
            <span
              className={`flag${status.bootloaderStatus === "unlocked" ? " flag--on" : ""}`}
              title="Status bootloader"
            >
              <Icon name="shield" size={14} />
              {status.bootloaderStatus === "unlocked" ? "Unlocked" : "Locked"}
            </span>
          )}
        </div>
      </div>
    </Card>
  );
}
