import { useMemo, useState } from "react";
import { useDevice } from "../hooks/useDevice.tsx";
import { useSession } from "../hooks/useSession.tsx";
import { useSettings } from "../hooks/useSettings.tsx";
import { useToast } from "../components/common/Toast";
import { formatBytes } from "../lib/format";
import * as api from "../lib/api";
import { PageHeader } from "../components/layout/PageHeader";
import { Card } from "../components/common/Card";
import { Button } from "../components/common/Button";
import { Icon } from "../components/common/Icon";
import { SessionPanel } from "../components/common/SessionPanel";
import { LogView } from "../components/common/LogView";
import { PartitionTable } from "../components/device/PartitionTable";
import { EmptyState } from "../components/common/EmptyState";

export function Backup() {
  const { status, partitions, isLoading, scan } = useDevice();
  const session = useSession();
  const { settings, update } = useSettings();
  const { push } = useToast();

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [pickingFolder, setPickingFolder] = useState(false);

  const selectedSize = useMemo(
    () =>
      partitions
        .filter((p) => selected.has(p.name))
        .reduce((sum, p) => sum + p.size, 0),
    [partitions, selected],
  );

  const canStart = selected.size > 0 && settings.workDir !== "" && status.mode === "unisoc_download";
  const isBusy = session.state === "running";

  const toggle = (name: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  const toggleAll = (names: string[]) => {
    setSelected((prev) => {
      const next = new Set(prev);
      const every = names.every((n) => next.has(n));
      for (const n of names) {
        if (every) next.delete(n);
        else next.add(n);
      }
      return next;
    });
  };

  const selectAll = () => setSelected(new Set(partitions.map((p) => p.name)));
  const clearAll = () => setSelected(new Set());

  const handlePickFolder = async () => {
    setPickingFolder(true);
    try {
      const dir = await api.pickFolder();
      if (dir != null && dir !== "") await update({ workDir: dir });
    } catch (err) {
      push("error", `Gagal memilih folder: ${String(err)}`);
    } finally {
      setPickingFolder(false);
    }
  };

  const handleStart = async () => {
    try {
      await session.startBackup({
        partitions: Array.from(selected),
        targetDir: settings.workDir,
        resume: settings.resumeEnabled,
      });
      push("info", "Backup dimulai. Perhatikan log di bawah.");
    } catch (err) {
      push("error", String(err));
    }
  };

  const handleCancel = async () => {
    await session.cancel();
    push("warn", "Backup dibatalkan. Gunakan tombol Lanjutkan untuk resume.");
  };

  return (
    <div className="view">
      <PageHeader
        title="Backup Firmware"
        subtitle="Baca partisi dari device ke folder tujuan"
        actions={
          <Button
            variant="secondary"
            size="sm"
            icon={<Icon name="scan" size={15} />}
            onClick={() => void scan()}
            disabled={isLoading}
          >
            Pindai Ulang
          </Button>
        }
      />

      {isBusy ? (
        <SessionPanel state={session.state} progress={session.progress} onCancel={() => void handleCancel()} />
      ) : (
        <>
          <Card
            title="Pilih Partisi"
            subtitle={`${partitions.length} partisi terdeteksi`}
            actions={
              <>
                <Button variant="ghost" size="sm" onClick={selectAll}>
                  Pilih Semua
                </Button>
                <Button variant="ghost" size="sm" onClick={clearAll}>
                  Bersihkan
                </Button>
              </>
            }
          >
            {partitions.length === 0 ? (
              <EmptyState
                icon="harddrive"
                title="Belum ada partisi"
                description="Sambungkan device dalam mode Download lalu tekan Pindai untuk membaca daftar partisi."
                action={
                  <Button variant="secondary" icon={<Icon name="scan" size={15} />} onClick={() => void scan()}>
                    Pindai Sekarang
                  </Button>
                }
              />
            ) : (
              <PartitionTable
                partitions={partitions}
                selected={selected}
                onToggle={toggle}
                onToggleAll={toggleAll}
                disabled={isBusy}
              />
            )}
          </Card>

          <Card
            title="Tujuan & Opsi"
            subtitle="Folder hasil backup dan pengaturan resume"
            actions={
              <Button
                variant="secondary"
                size="sm"
                icon={<Icon name="folder" size={15} />}
                onClick={() => void handlePickFolder()}
                disabled={pickingFolder}
              >
                {pickingFolder ? "Memilih..." : "Pilih Folder"}
              </Button>
            }
          >
            <div className="field-row">
              <label className="field">
                <span className="field__label">Folder tujuan</span>
                <code className="field__value">{settings.workDir || "-"}</code>
              </label>
            </div>

            <div className="backup-options">
              <label className="opt">
                <input
                  type="checkbox"
                  className="checkbox"
                  checked={settings.resumeEnabled}
                  disabled={isBusy}
                  onChange={(e) => void update({ resumeEnabled: e.currentTarget.checked })}
                />
                <span>
                  <strong>Resume otomatis</strong>
                  <small>Lewati partisi yang sudah lengkap bila dump terputus.</small>
                </span>
              </label>
            </div>

            <div className="backup-footer">
              <div className="backup-footer__info">
                <span>
                  {selected.size} dari {partitions.length} partisi dipilih
                </span>
                <span className="mono">{formatBytes(selectedSize)}</span>
              </div>
              <Button
                variant="primary"
                size="lg"
                icon={<Icon name="play" size={16} />}
                onClick={() => void handleStart()}
                disabled={!canStart}
                title={
                  status.mode !== "unisoc_download"
                    ? "Masuk mode Download dahulu (power off, Volume Up+Down, colok USB)"
                    : undefined
                }
              >
                Mulai Backup
              </Button>
            </div>

            {status.mode !== "unisoc_download" && (
              <p className="form-hint">
                Device belum dalam mode Download. Backup hanya tersedia pada mode Download.
              </p>
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
