import type { AppMode, View } from "../../types/view";
import { Icon, type IconName } from "../common/Icon";

interface NavItem {
  id: View;
  label: string;
  icon: IconName;
  advanced?: boolean;
}

interface NavGroup {
  label?: string;
  items: NavItem[];
}

const NAV_GROUPS: NavGroup[] = [
  {
    label: "Utama",
    items: [{ id: "dashboard", label: "Dasbor", icon: "dashboard" }],
  },
  {
    label: "Operasi",
    items: [
      { id: "backup", label: "Backup", icon: "backup" },
      { id: "restore", label: "Restore", icon: "restore" },
    ],
  },
  {
    label: "Teknis",
    items: [{ id: "device", label: "Informasi Perangkat", icon: "chip", advanced: true }],
  },
  {
    items: [{ id: "settings", label: "Pengaturan", icon: "settings" }],
  },
];

interface SidebarProps {
  view: View;
  appMode: AppMode;
  onNavigate: (view: View) => void;
  onModeChange: (mode: AppMode) => void;
  version: string;
}

export function Sidebar({ view, appMode, onNavigate, onModeChange, version }: SidebarProps) {
  const visibleGroups = NAV_GROUPS.map((group) => ({
    ...group,
    items: group.items.filter((item) => item.advanced !== true || appMode === "advanced"),
  })).filter((group) => group.items.length > 0);

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__logo">
          <Icon name="chip" size={22} />
        </div>
        <div className="sidebar__brand-text">
          <span className="sidebar__title">SPD Service</span>
          <span className="sidebar__subtitle">Android Tool</span>
        </div>
      </div>

      <div className="mode-switch" role="tablist" aria-label="Mode aplikasi">
        <button
          type="button"
          role="tab"
          aria-selected={appMode === "basic"}
          className={`mode-switch__btn${appMode === "basic" ? " mode-switch__btn--active" : ""}`}
          onClick={() => onModeChange("basic")}
        >
          Basic
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={appMode === "advanced"}
          className={`mode-switch__btn${appMode === "advanced" ? " mode-switch__btn--active" : ""}`}
          onClick={() => onModeChange("advanced")}
        >
          Advanced
        </button>
      </div>

      <nav className="sidebar__nav" aria-label="Navigasi utama">
        {visibleGroups.map((group, gi) => (
          <div key={gi} className="nav-group">
            {group.label != null && <span className="nav-group__label">{group.label}</span>}
            {group.items.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`nav-item${view === item.id ? " nav-item--active" : ""}`}
                onClick={() => onNavigate(item.id)}
                aria-current={view === item.id ? "page" : undefined}
              >
                <Icon name={item.icon} size={18} />
                <span>{item.label}</span>
                {item.advanced === true && appMode === "advanced" && (
                  <span className="nav-item__badge">Adv</span>
                )}
              </button>
            ))}
          </div>
        ))}
      </nav>

      <div className="sidebar__footer">
        <span className="sidebar__version">v{version}</span>
      </div>
    </aside>
  );
}
