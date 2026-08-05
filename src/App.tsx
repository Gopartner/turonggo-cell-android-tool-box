import { useEffect, useState } from "react";
import { DeviceProvider } from "./hooks/useDevice.tsx";
import { SessionProvider } from "./hooks/useSession.tsx";
import { SettingsProvider } from "./hooks/useSettings.tsx";
import { ToastProvider } from "./components/common/Toast";
import { Sidebar } from "./components/layout/Sidebar";
import { StatusBar } from "./components/layout/StatusBar";
import { Dashboard } from "./views/Dashboard";
import { Backup } from "./views/Backup";
import { Restore } from "./views/Restore";
import { DeviceInfoView } from "./views/DeviceInfo";
import { Settings } from "./views/Settings";
import type { AppMode, View } from "./types/view";
import * as api from "./lib/api";
import "./App.css";

const ADVANCED_ONLY: ReadonlySet<View> = new Set(["device"]);

function App() {
  const [view, setView] = useState<View>("dashboard");
  const [appMode, setAppMode] = useState<AppMode>("basic");
  const [version, setVersion] = useState("0.1.0");

  useEffect(() => {
    api
      .appVersion()
      .then(setVersion)
      .catch(() => {});
  }, []);

  const handleNavigate = (next: View) => {
    if (ADVANCED_ONLY.has(next) && appMode !== "advanced") return;
    setView(next);
  };

  const handleModeChange = (mode: AppMode) => {
    setAppMode(mode);
    if (mode === "basic" && ADVANCED_ONLY.has(view)) setView("dashboard");
  };

  return (
    <ToastProvider>
      <SettingsProvider>
        <DeviceProvider>
          <SessionProvider>
            <div className="app-shell">
              <Sidebar
                view={view}
                appMode={appMode}
                onNavigate={handleNavigate}
                onModeChange={handleModeChange}
                version={version}
              />
              <div className="app-main">
                <div className="app-content">
                  {view === "dashboard" && <Dashboard onNavigate={setView} />}
                  {view === "backup" && <Backup />}
                  {view === "restore" && <Restore />}
                  {view === "device" && appMode === "advanced" && <DeviceInfoView />}
                  {view === "settings" && <Settings />}
                </div>
                <StatusBar />
              </div>
            </div>
          </SessionProvider>
        </DeviceProvider>
      </SettingsProvider>
    </ToastProvider>
  );
}

export default App;
