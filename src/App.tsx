import { useTrafficFeed } from "./hooks/useTrafficFeed";
import { useAlerts } from "./hooks/useAlerts";
import { useSettings } from "./hooks/useSettings";
import { useTrafficStore, ViewType } from "./store/trafficStore";
import LiveTable from "./components/Dashboard/LiveTable";
import BandwidthGraph from "./components/Dashboard/BandwidthGraph";
import AppSummary from "./components/Dashboard/AppSummary";
import HistoryView from "./components/History/HistoryView";
import GeoMap from "./components/Map/GeoMap";
import AlertsPanel from "./components/Alerts/AlertsPanel";
import DNSLeakView from "./components/DNS/DNSLeakView";
import SettingsPanel from "./components/Settings/SettingsPanel";
import StatCard from "./components/shared/StatCard";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}

const navItems: Array<{ id: ViewType; label: string; icon: string }> = [
  { id: "dashboard", label: "Dashboard", icon: "📊" },
  { id: "history", label: "History", icon: "📋" },
  { id: "map", label: "World Map", icon: "🌍" },
  { id: "alerts", label: "Alerts", icon: "🔔" },
  { id: "dns", label: "DNS Log", icon: "🔍" },
  { id: "settings", label: "Settings", icon: "⚙️" },
];

export default function App() {
  useTrafficFeed();
  useAlerts();
  useSettings();

  const currentView = useTrafficStore((s) => s.currentView);
  const setCurrentView = useTrafficStore((s) => s.setCurrentView);
  const speedStats = useTrafficStore((s) => s.speedStats);
  const liveConnections = useTrafficStore((s) => s.liveConnections);
  const alerts = useTrafficStore((s) => s.alerts);
  const theme = useTrafficStore((s) => s.theme);
  const setTheme = useTrafficStore((s) => s.setTheme);
  const captureRunning = useTrafficStore((s) => s.captureRunning);

  const undismissedAlerts = alerts.filter((a) => !a.dismissed).length;

  return (
    <div className="flex h-screen overflow-hidden bg-surface">
      {/* Sidebar */}
      <aside className="w-56 flex flex-col border-r border-panel-border bg-surface-300/50 shrink-0">
        <div className="p-4 flex items-center gap-3 border-b border-panel-border">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent-cyan to-blue-600 flex items-center justify-center text-sm font-bold">
            NP
          </div>
          <div>
            <h1 className="text-sm font-semibold text-white">NetPulse</h1>
            <p className="text-[10px] text-gray-500">
              {captureRunning ? (
                <span className="text-accent-emerald">● Live</span>
              ) : (
                <span className="text-gray-500">○ Idle</span>
              )}
            </p>
          </div>
        </div>

        <nav className="flex-1 py-2 px-2 space-y-0.5">
          {navItems.map((item) => (
            <button
              key={item.id}
              id={`nav-${item.id}`}
              onClick={() => setCurrentView(item.id)}
              className={`nav-item w-full text-left text-sm ${
                currentView === item.id ? "active" : ""
              }`}
            >
              <span className="text-base">{item.icon}</span>
              <span>{item.label}</span>
              {item.id === "alerts" && undismissedAlerts > 0 && (
                <span className="ml-auto bg-accent-red text-white text-[10px] px-1.5 py-0.5 rounded-full min-w-[18px] text-center">
                  {undismissedAlerts}
                </span>
              )}
            </button>
          ))}
        </nav>

        <div className="p-3 border-t border-panel-border">
          <div className="glass-panel p-2 rounded-lg text-[11px] space-y-1">
            <div className="flex justify-between items-center">
              <span className="text-gray-500">↑ Upload</span>
              <span className="font-mono text-accent-cyan">
                {formatSpeed(speedStats.upload_speed)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-500">↓ Download</span>
              <span className="font-mono text-accent-emerald">
                {formatSpeed(speedStats.download_speed)}
              </span>
            </div>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Top Bar */}
        <header className="h-12 flex items-center justify-between px-4 border-b border-panel-border bg-surface-300/30 shrink-0">
          <div className="flex items-center gap-4">
            <StatCard
              label="Upload"
              value={`↑ ${formatSpeed(speedStats.upload_speed)}`}
              color="cyan"
            />
            <StatCard
              label="Download"
              value={`↓ ${formatSpeed(speedStats.download_speed)}`}
              color="emerald"
            />
            <StatCard
              label="Connections"
              value={liveConnections.size.toString()}
              color="white"
            />
          </div>

          <div className="flex items-center gap-3">
            {undismissedAlerts > 0 && (
              <button
                onClick={() => setCurrentView("alerts")}
                className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-accent-amber/10 text-accent-amber text-xs hover:bg-accent-amber/20 transition-colors"
                id="alerts-badge"
              >
                🔔 {undismissedAlerts}
              </button>
            )}
            <button
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              className="p-1.5 rounded-lg hover:bg-white/5 transition-colors text-sm"
              id="theme-toggle"
              title="Toggle theme"
            >
              {theme === "dark" ? "☀️" : "🌙"}
            </button>
          </div>
        </header>

        {/* View Content */}
        <div className="flex-1 overflow-auto p-4">
          {currentView === "dashboard" && <DashboardView />}
          {currentView === "history" && <HistoryView />}
          {currentView === "map" && <GeoMap />}
          {currentView === "alerts" && <AlertsPanel />}
          {currentView === "dns" && <DNSLeakView />}
          {currentView === "settings" && <SettingsPanel />}
        </div>
      </main>
    </div>
  );
}

function DashboardView() {
  return (
    <div className="space-y-4 h-full flex flex-col">
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-4">
        <div className="xl:col-span-2 glass-panel p-4 min-h-[250px]">
          <h3 className="text-sm font-medium text-gray-400 mb-3">
            Bandwidth (Last 60s)
          </h3>
          <BandwidthGraph />
        </div>
        <div className="glass-panel p-4 min-h-[250px] overflow-auto">
          <h3 className="text-sm font-medium text-gray-400 mb-3">
            App Summary
          </h3>
          <AppSummary />
        </div>
      </div>
      <div className="flex-1 glass-panel p-4 min-h-[300px]">
        <h3 className="text-sm font-medium text-gray-400 mb-3">
          Active Connections
        </h3>
        <LiveTable />
      </div>
    </div>
  );
}
