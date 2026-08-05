import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { AppSettings } from "../types/ipc";
import * as api from "../lib/api";

interface SettingsContextValue {
  settings: AppSettings;
  isLoading: boolean;
  update: (patch: Partial<AppSettings>) => Promise<void>;
  save: (next: AppSettings) => Promise<void>;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

const DEFAULT_SETTINGS: AppSettings = {
  chipset: "",
  workDir: "",
  resumeEnabled: true,
  allowRestoreToStock: false,
};

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    api
      .getSettings()
      .then((s) => setSettings(s))
      .catch(() => {
        // default dipakai; kesalahan muncul saat user menabung
      })
      .finally(() => setIsLoading(false));
  }, []);

  const save = useCallback(async (next: AppSettings) => {
    await api.setSettings(next);
    setSettings(next);
  }, []);

  const update = useCallback(
    async (patch: Partial<AppSettings>) => {
      const next = { ...settings, ...patch };
      await save(next);
    },
    [settings, save],
  );

  const value = useMemo(() => ({ settings, isLoading, update, save }), [settings, isLoading, update, save]);

  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}

export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) throw new Error("useSettings harus dipakai di dalam SettingsProvider");
  return ctx;
}
