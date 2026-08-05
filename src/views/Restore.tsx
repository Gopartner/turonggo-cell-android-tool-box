import { useState } from "react";
import { useDevice } from "../hooks/useDevice.tsx";
import { useSession } from "../hooks/useSession.tsx";
import { useSettings } from "../hooks/useSettings.tsx";
import { useToast } from "../components/common/Toast";
import { formatBytes } from "../lib/format";
import * as api from "../lib/api";
import type { BackupManifest } from "../types/ipc";
import { PageHeader } from "../components/layout/PageHeader";
import { Card } from "../components/common/Card";
import { Button } from "../components/common/Button";
import { Icon } from "../components/common/Icon";
import { SessionPanel } from "../components/common/SessionPanel";
import { LogView } from "../components/common/LogView";
import { Table, type TableColumn } from "../components/common/Table";
import { EmptyState } from "../components/common/EmptyState";
import { ModeBadge } from "../components/device/ModeBadge";

export function Restore() {
  const { status } = useDevice();
  const session = useSession();
  const { settings } = useSettings();
  const { push } = useToast();

  const [manifest, setManifest] = useState<BackupManifest | null>(null);
  const [folder, setFolder] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirmText, setConfirmText] = useState("");

  const chipsetMatch = manifest != null && status.chip != null && manifest.chipset === status.chip;
  const stockGated = manifest != null && !settings.allowRestoreToStock;
  const confirmOk = !stockGated || confirmText === "LANJUT";
  const canStart = manifest != null && chipsetMatch && confirmOk && status.mode === "unisoc_download";
  const isBusy = session.state === "running";

  const handleLoad = async () => {
    setLoading(true);
    try {
      const dir = await api.pickFolder();
      if (dir == null || dir === "") return;
      const m = await api.readBackupFolder(dir);
      setFolder(dir);
      setManifest(m);
      setConfirmText("");
      push("info", `Manifest dibaca: ${m.partitions.length} partisi untuk chipset ${m.chipset}.`);
    } catch (err) {
      push("error", `Gagal membaca manifest: ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStart = async () => {
    if (manifest == null) return;
    try {
      await session.startRestore({
        partitions: manifest.partitions.map((p) => ({ name: p.name, file: p.file })),
      });
      push("info", "Restore dimulai. Jangan putuskan USB.");
    } catch (err) {
      push("error", String(err));
    }
  };

  const handleCancel = async () => {
    await session.cancel();
    push("warn", "Restore dibatalkan pada posisi aman terakhir.");
  };

  const columns: TableColumn<BackupManifest["partitions"][number]>[] = [
    { key: "name", header: "Partisi" },
    { key: "file", header: "File", render: (row) => <code className="mono">{row.file}</code> },
    {
      key: "size",
      header: "Ukuran",
      render: (row) => <span className="mono">{formatBytes(row.size)}</span>,
    },
  ];

  return (
    <div className="view">
      <PageHeader
        title="Restore Firmware"
        subtitle="Tulis kembali partisi dari hasil backup ke device"
        actions={
          <Button
            variant="secondary"
            size="sm"
            icon={<Icon name="folder" size={15} />}
            onClick={() => void handleLoad()}
            disabled={loading || isBusy}
          >
            {loading ? "Membaca..." : "Pilih Folder Backup"}
          </Button>
        }
      />

      {isBusy ? (
        <SessionPanel state={session.state} progress={session.progress} onCancel={() => void handleCancel()} />
      ) : manifest == null ? (
        <Card title="Pilih Hasil Backup">
          <EmptyState
            icon="restore"
            title="Belum ada manifest"
            description="Pilih folder yang berisi hasil backup (manifest + file partisi .img) untuk ditampilkan di sini."
            action={
              <Button variant="primary" icon={<Icon name="folder" size={16} />} onClick={() => void handleLoad()}>
                Pilih Folder Backup
              </Button>
            }
          />
        </Card>
      ) : (
        <>
          <Card
            title="Manifest Backup"
            subtitle={folder ?? undefined}
            actions={<ModeBadge mode={status.mode} />}
          >
            <div className="manifest-row">
              <span className="manifest-row__label">Chipset</span>
              <span className="manifest-row__value">
                {manifest.chipset}
                <span className={`chip-match${chipsetMatch ? " chip-match--ok" : " chip-match--bad"}`}>
                  {chipsetMatch
                    ? "Cocok dengan device"
                    : status.chip != null
                      ? `Tidak cocok (device: ${status.chip})`
                      : "Device belum terdeteksi"}
                </span>
              </span>
            </div>
            <Table
              columns={columns}
              rows={manifest.partitions}
              rowKey={(p) => p.name}
              emptyText="Tidak ada partisi di manifest."
            />
          </Card>

          {stockGated && (
            <div className="restore-gate">
              <div className="restore-gate__head">
                <Icon name="warning" size={20} />
                <h4>Peringatan: menimpa partisi sistem/stock</h4>
              </div>
              <p>
                Menulis ke partisi sistem dapat membuat device tidak bisa boot bila file berasal dari
                device lain atau sudah dimodifikasi. Pastikan backup ini dibuat dari{" "}
                <strong>device yang sama</strong>.
              </p>
              <label className="restore-gate__confirm">
                Ketik <code>LANJUT</code> untuk mengizinkan:
                <input
                  type="text"
                  className="input"
                  value={confirmText}
                  onChange={(e) => setConfirmText(e.currentTarget.value)}
                  placeholder="LANJUT"
                  autoComplete="off"
                  spellCheck={false}
                />
              </label>
            </div>
          )}

          <Card title="Mulai Restore">
            <div className="backup-footer">
              <div className="backup-footer__info">
                <span>
                  {manifest.partitions.length} partisi siap ditulis
                  {settings.allowRestoreToStock ? " · gating dinonaktifkan" : " · butuh konfirmasi"}
                </span>
              </div>
              <Button
                variant="danger"
                size="lg"
                icon={<Icon name="play" size={16} />}
                onClick={() => void handleStart()}
                disabled={!canStart}
              >
                Mulai Restore
              </Button>
            </div>
            {status.mode !== "unisoc_download" && (
              <p className="form-hint">Restore membutuhkan mode Download (power off, Volume Up+Down, colok USB).</p>
            )}
          </Card>
        </>
      )}

      <Card title="Log" subtitle="Output langsung dari backend">
        <LogView entries={session.logs} maxHeight={300} />
      </Card>
    </div>
  );
}
