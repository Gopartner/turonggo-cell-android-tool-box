import { useDevice } from "../hooks/useDevice.tsx";
import { useSession } from "../hooks/useSession.tsx";
import { formatBytes, formatDuration } from "../lib/format";
import { DeviceCard } from "../components/device/DeviceCard";
import { Card } from "../components/common/Card";
import { LogView } from "../components/common/LogView";
import { Icon } from "../components/common/Icon";
import { PageHeader } from "../components/layout/PageHeader";
import { Button } from "../components/common/Button";
import type { View } from "../types/view";

function StatCard({
  label,
  value,
  icon,
  tone = "default",
}: {
  label: string;
  value: string;
  icon: "harddrive" | "chip" | "clock";
  tone?: "default" | "ok" | "warn";
}) {
  return (
    <div className={`stat-card stat-card--${tone}`}>
      <div className="stat-card__icon">
        <Icon name={icon} size={18} />
      </div>
      <div className="stat-card__body">
        <span className="stat-card__label">{label}</span>
        <span className="stat-card__value">{value}</span>
      </div>
    </div>
  );
}

interface DashboardProps {
  onNavigate: (view: View) => void;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const { partitions, status } = useDevice();
  const { logs, summary, state } = useSession();

  const totalBytes = partitions.reduce((sum, p) => sum + p.size, 0);
  const userBytes = partitions
    .filter((p) => p.isUserPartition === true)
    .reduce((sum, p) => sum + p.size, 0);

  return (
    <div className="view">
      <PageHeader
        title="Dasbor"
        subtitle="Ringkasan perangkat dan aktivitas terakhir"
        actions={
          <Button
            variant="primary"
            icon={<Icon name="backup" size={16} />}
            onClick={() => onNavigate("backup")}
          >
            Mulai Backup
          </Button>
        }
      />

      <div className="grid grid--3">
        <StatCard
          label="Partisi terdeteksi"
          value={partitions.length > 0 ? `${partitions.length}` : "-"}
          icon="harddrive"
        />
        <StatCard
          label="Total ukuran"
          value={totalBytes > 0 ? formatBytes(totalBytes) : "-"}
          icon="chip"
        />
        <StatCard
          label="Data pribadi (userdata)"
          value={userBytes > 0 ? formatBytes(userBytes) : "-"}
          icon="clock"
          tone="warn"
        />
      </div>

      <div className="grid grid--dash">
        <DeviceCard />

        <Card
          title="Aktivitas Terakhir"
          subtitle={summary != null ? `Durasi ${formatDuration(summary.durationMs)}` : "Belum ada sesi selesai"}
          actions={
            state !== "idle" ? (
              <Button variant="ghost" size="sm" onClick={() => onNavigate("backup")}>
                Buka detail
              </Button>
            ) : undefined
          }
        >
          <div className="dash-logs">
            <LogView entries={logs.slice(-40)} maxHeight={280} />
          </div>
        </Card>
      </div>

      {status.mode === "fastboot" || status.mode === "adb" ? (
        <div className="notice notice--warn">
          <Icon name="warning" size={18} />
          <p>
            Mode <strong>{status.mode}</strong> tidak mendukung operasi read/write firmware. Matikan
            device, tekan <em>Volume Up + Volume Down</em>, lalu sambungkan USB untuk masuk mode
            Download.
          </p>
        </div>
      ) : null}
    </div>
  );
}
