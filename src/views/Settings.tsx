import { useEffect, useState } from "react";
import { useSettings } from "../hooks/useSettings.tsx";
import { useToast } from "../components/common/Toast";
import * as api from "../lib/api";
import { PageHeader } from "../components/layout/PageHeader";
import { Card } from "../components/common/Card";
import { Button } from "../components/common/Button";
import { Icon } from "../components/common/Icon";
import { Toggle } from "../components/common/Toggle";

const KNOWN_CHIPSETS = ["ums9230", "ums5120", "ums9621", "ums9117"];

export function Settings() {
  const { settings, isLoading, update, save } = useSettings();
  const { push } = useToast();
  const [chipset, setChipset] = useState(settings.chipset);
  const [version, setVersion] = useState("0.1.0");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setChipset(settings.chipset);
  }, [settings.chipset]);

  useEffect(() => {
    api
      .appVersion()
      .then(setVersion)
      .catch(() => {});
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      await save({ ...settings, chipset });
      push("success", "Pengaturan disimpan.");
    } catch (err) {
      push("error", `Gagal menyimpan: ${String(err)}`);
    } finally {
      setSaving(false);
    }
  };

  const handlePickWorkDir = async () => {
    try {
      const dir = await api.pickFolder();
      if (dir != null && dir !== "") await update({ workDir: dir });
    } catch (err) {
      push("error", String(err));
    }
  };

  return (
    <div className="view">
      <PageHeader
        title="Pengaturan"
        subtitle="Konfigurasi chipset, folder kerja, dan gating keamanan"
        actions={
          <Button
            variant="primary"
            size="sm"
            icon={<Icon name="check" size={15} />}
            onClick={() => void handleSave()}
            disabled={isLoading || saving}
          >
            Simpan
          </Button>
        }
      />

      <Card title="Chipset" subtitle="Chipset aktif untuk operasi Download">
        <label className="field">
          <span className="field__label">Chipset (SoC)</span>
          <input
            type="text"
            className="input"
            list="chipset-options"
            value={chipset}
            disabled={isLoading}
            onChange={(e) => setChipset(e.currentTarget.value)}
            placeholder="mis. ums9230"
            autoComplete="off"
            spellCheck={false}
          />
          <datalist id="chipset-options">
            {KNOWN_CHIPSETS.map((c) => (
              <option key={c} value={c} />
            ))}
          </datalist>
          <small className="field__hint">Chipset menentukan FDL dan EXEC_ADDR yang dipakai.</small>
        </label>
      </Card>

      <Card
        title="Folder Kerja"
        subtitle="Direktori default untuk hasil backup"
        actions={
          <Button variant="secondary" size="sm" icon={<Icon name="folder" size={15} />} onClick={() => void handlePickWorkDir()}>
            Pilih Folder
          </Button>
        }
      >
        <label className="field">
          <span className="field__label">Direktori tujuan</span>
          <code className="field__value">{settings.workDir || "-"}</code>
        </label>
      </Card>

      <Card title="Opsi Operasi" subtitle="Perilaku backup dan restore">
        <div className="toggle-list">
          <Toggle
            checked={settings.resumeEnabled}
            onChange={(v) => void update({ resumeEnabled: v })}
            label="Resume otomatis"
            description="Lewati partisi yang sudah lengkap bila dump terputus di tengah jalan."
          />
          <div className="toggle-list__danger">
            <Toggle
              checked={settings.allowRestoreToStock}
              onChange={(v) => void update({ allowRestoreToStock: v })}
              label="Izinkan restore ke partisi stock"
              description="Menonaktifkan konfirmasi LANJUT di layar Restore. Berbahaya bila backup bukan dari device yang sama."
            />
          </div>
        </div>
      </Card>

      <Card title="Tentang" subtitle="Informasi aplikasi">
        <div className="about-row">
          <span className="about-row__label">Versi</span>
          <span className="about-row__value">v{version}</span>
        </div>
        <div className="about-row">
          <span className="about-row__label">Runtime</span>
          <span className="about-row__value">
            {api.isTauriRuntime() ? "Tauri (desktop)" : "Preview browser (mock)"}
          </span>
        </div>
        <p className="form-hint">
          SPD Backup Tool — backup &amp; restore firmware perangkat Unisoc/SPRD.
        </p>
      </Card>
    </div>
  );
}
