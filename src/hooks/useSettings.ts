import { useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTrafficStore } from "../store/trafficStore";

export interface SettingsData {
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

export function useSettings() {
  const setTheme = useTrafficStore((s) => s.setTheme);

  const loadSettings = useCallback(async (): Promise<SettingsData | null> => {
    try {
      const settings = await invoke<SettingsData>("get_settings");
      if (settings.theme === "light" || settings.theme === "dark") {
        setTheme(settings.theme);
      }
      return settings;
    } catch (err) {
      console.error("Failed to load settings:", err);
      return null;
    }
  }, [setTheme]);

  const updateSetting = useCallback(async (key: string, value: string) => {
    try {
      await invoke("update_setting", { key, value });
    } catch (err) {
      console.error("Failed to update setting:", err);
    }
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return { loadSettings, updateSetting };
}
