import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SettingsData {
  capture_interface: string;
  theme: string;
  trusted_countries: string[];
  alert_odd_hours: boolean;
  alert_unknown_process: boolean;
  alert_foreign_country: boolean;
  data_retention_days: number;
  notification_enabled: boolean;
  start_minimized: boolean;
  auto_start_capture: boolean;
}

export default function SettingsPanel() {
  const [settings, setSettings] = useState<SettingsData | null>(null);
  const [interfaces, setInterfaces] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [countriesInput, setCountriesInput] = useState("");

  const loadSettings = useCallback(async () => {
    try {
      const [settingsData, ifaces] = await Promise.all([
        invoke<SettingsData>("get_settings"),
        invoke<string[]>("get_interfaces"),
      ]);
      setSettings(settingsData);
      setInterfaces(ifaces);
      setCountriesInput(settingsData.trusted_countries.join(", "));
    } catch (err) {
      console.error("Failed to load settings:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const saveSetting = async (key: string, value: string) => {
    setSaving(true);
    try {
      await invoke("update_setting", { key, value });
    } catch (err) {
      console.error("Failed to save setting:", err);
    } finally {
      setSaving(false);
    }
  };

  const handleThemeChange = async (theme: string) => {
    await saveSetting("theme", theme);
    document.documentElement.classList.toggle("light", theme === "light");
    document.documentElement.classList.toggle("dark", theme === "dark");
    setSettings((prev) => (prev ? { ...prev, theme } : null));
  };

  const handleToggle = async (key: string, value: boolean) => {
    await saveSetting(key, value.toString());
    setSettings((prev) =>
      prev ? { ...prev, [key]: value } : null
    );
  };

  if (loading || !settings) {
    return (
      <div className="flex items-center justify-center h-48">
        <div className="text-gray-500">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <h2 className="text-lg font-semibold">Settings</h2>

      {/* Capture Interface */}
      <Section title="Network Capture" icon="📡">
        <div className="space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">
              Capture Interface
            </label>
            <select
              value={settings.capture_interface}
              onChange={async (e) => {
                const value = e.target.value;
                await saveSetting("capture_interface", value);
                setSettings({ ...settings, capture_interface: value });
                try {
                  await invoke("set_capture_interface", { name: value });
                } catch (err) {
                  console.error("Failed to set interface:", err);
                }
              }}
              className="w-full px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-white focus:outline-none focus:border-accent-cyan/50"
              id="setting-interface"
            >
              <option value="auto">Auto-detect</option>
              {interfaces.map((iface) => (
                <option key={iface} value={iface}>
                  {iface}
                </option>
              ))}
            </select>
          </div>

          <Toggle
            label="Auto-start capture on launch"
            checked={settings.auto_start_capture}
            onChange={(v) => handleToggle("auto_start_capture", v)}
          />
        </div>
      </Section>

      {/* Appearance */}
      <Section title="Appearance" icon="🎨">
        <div>
          <label className="text-xs text-gray-400 block mb-2">Theme</label>
          <div className="flex gap-2">
            <button
              onClick={() => handleThemeChange("dark")}
              className={`flex-1 py-2 rounded-lg text-sm transition-all ${
                settings.theme === "dark"
                  ? "bg-accent-cyan/10 border border-accent-cyan/30 text-accent-cyan"
                  : "bg-white/5 border border-white/10 text-gray-400 hover:bg-white/10"
              }`}
              id="theme-dark"
            >
              🌙 Dark
            </button>
            <button
              onClick={() => handleThemeChange("light")}
              className={`flex-1 py-2 rounded-lg text-sm transition-all ${
                settings.theme === "light"
                  ? "bg-accent-cyan/10 border border-accent-cyan/30 text-accent-cyan"
                  : "bg-white/5 border border-white/10 text-gray-400 hover:bg-white/10"
              }`}
              id="theme-light"
            >
              ☀️ Light
            </button>
          </div>
        </div>
      </Section>

      {/* Alert Settings */}
      <Section title="Alert Rules" icon="🔔">
        <div className="space-y-3">
          <Toggle
            label="Flag connections between 1am–5am"
            checked={settings.alert_odd_hours}
            onChange={(v) => handleToggle("alert_odd_hours", v)}
          />
          <Toggle
            label="Flag unknown/uncommon processes"
            checked={settings.alert_unknown_process}
            onChange={(v) => handleToggle("alert_unknown_process", v)}
          />
          <Toggle
            label="Flag connections to non-trusted countries"
            checked={settings.alert_foreign_country}
            onChange={(v) => handleToggle("alert_foreign_country", v)}
          />
          <Toggle
            label="Enable desktop notifications"
            checked={settings.notification_enabled}
            onChange={(v) => handleToggle("notification_enabled", v)}
          />
        </div>
      </Section>

      {/* Trusted Countries */}
      <Section title="Trusted Countries" icon="🌍">
        <div>
          <label className="text-xs text-gray-400 block mb-1">
            Country codes (comma-separated, e.g. US, GB, DE)
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={countriesInput}
              onChange={(e) => setCountriesInput(e.target.value)}
              className="flex-1 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-accent-cyan/50"
              id="setting-countries"
            />
            <button
              onClick={async () => {
                const countries = countriesInput
                  .split(",")
                  .map((c) => c.trim().toUpperCase())
                  .filter((c) => c.length === 2);
                await saveSetting(
                  "trusted_countries",
                  JSON.stringify(countries)
                );
                setSettings({
                  ...settings,
                  trusted_countries: countries,
                });
              }}
              className="btn-primary text-xs"
              disabled={saving}
            >
              Save
            </button>
          </div>
          <div className="flex flex-wrap gap-1 mt-2">
            {settings.trusted_countries.map((code) => (
              <span
                key={code}
                className="px-1.5 py-0.5 rounded bg-accent-cyan/10 text-accent-cyan text-[10px] font-mono"
              >
                {code}
              </span>
            ))}
          </div>
        </div>
      </Section>

      {/* Data Management */}
      <Section title="Data Management" icon="💾">
        <div className="space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">
              Data retention period (days)
            </label>
            <input
              type="number"
              value={settings.data_retention_days}
              onChange={async (e) => {
                const days = parseInt(e.target.value, 10);
                if (!isNaN(days) && days > 0) {
                  await saveSetting("data_retention_days", days.toString());
                  setSettings({ ...settings, data_retention_days: days });
                }
              }}
              className="w-32 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-white focus:outline-none focus:border-accent-cyan/50"
              min={1}
              max={365}
              id="setting-retention"
            />
          </div>
          <Toggle
            label="Start minimized to system tray"
            checked={settings.start_minimized}
            onChange={(v) => handleToggle("start_minimized", v)}
          />
        </div>
      </Section>

      {/* About */}
      <Section title="About" icon="ℹ️">
        <div className="text-xs text-gray-400 space-y-1">
          <p>
            <span className="text-gray-500">Version:</span>{" "}
            <span className="text-white">1.0.0</span>
          </p>
          <p>
            <span className="text-gray-500">License:</span>{" "}
            <span className="text-white">MIT</span>
          </p>
          <p className="text-gray-600 mt-2">
            NetPulse — Real-time network traffic visualizer
          </p>
        </div>
      </Section>
    </div>
  );
}

function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon: string;
  children: React.ReactNode;
}) {
  return (
    <div className="glass-panel p-4">
      <h3 className="text-sm font-medium text-white mb-3 flex items-center gap-2">
        <span>{icon}</span>
        {title}
      </h3>
      {children}
    </div>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between cursor-pointer group">
      <span className="text-xs text-gray-300 group-hover:text-white transition-colors">
        {label}
      </span>
      <button
        onClick={() => onChange(!checked)}
        className={`relative w-9 h-5 rounded-full transition-colors ${
          checked ? "bg-accent-cyan" : "bg-white/10"
        }`}
      >
        <div
          className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform ${
            checked ? "translate-x-4" : "translate-x-0.5"
          }`}
        />
      </button>
    </label>
  );
}
