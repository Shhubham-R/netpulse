import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTrafficStore, AlertRecord } from "../../store/trafficStore";

const severityConfig: Record<string, { icon: string; bg: string; border: string; text: string }> = {
  danger: {
    icon: "🚨",
    bg: "bg-red-500/5",
    border: "border-red-500/20",
    text: "text-red-400",
  },
  warning: {
    icon: "⚠️",
    bg: "bg-amber-500/5",
    border: "border-amber-500/20",
    text: "text-amber-400",
  },
  info: {
    icon: "ℹ️",
    bg: "bg-blue-500/5",
    border: "border-blue-500/20",
    text: "text-blue-400",
  },
};

const ruleLabels: Record<string, string> = {
  odd_hours: "Odd Hours Activity",
  unknown_process: "Unknown Process",
  foreign_country: "Foreign Country",
  high_volume_unknown: "High Volume (Unknown)",
  tor_exit_node: "Tor Exit Node",
  first_connection: "First Connection",
  blocked_process_attempt: "Blocked Process Attempt",
};

function formatTime(iso: string): string {
  try {
    const date = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  } catch {
    return iso;
  }
}

export default function AlertsPanel() {
  const alerts = useTrafficStore((s) => s.alerts);
  const setAlerts = useTrafficStore((s) => s.setAlerts);
  const [showDismissed, setShowDismissed] = useState(false);

  const loadAlerts = useCallback(async () => {
    try {
      const result = await invoke<AlertRecord[]>("get_alerts", {
        dismissed: showDismissed,
      });
      setAlerts(result);
    } catch (err) {
      console.error("Failed to load alerts:", err);
    }
  }, [showDismissed, setAlerts]);

  useEffect(() => {
    loadAlerts();
  }, [loadAlerts]);

  const handleDismiss = async (id: number) => {
    try {
      await invoke("dismiss_alert", { id });
      setAlerts(alerts.map((a) => (a.id === id ? { ...a, dismissed: true } : a)));
    } catch (err) {
      console.error("Failed to dismiss alert:", err);
    }
  };

  const handleDismissAll = async () => {
    try {
      for (const alert of alerts.filter((a) => !a.dismissed)) {
        await invoke("dismiss_alert", { id: alert.id });
      }
      setAlerts(alerts.map((a) => ({ ...a, dismissed: true })));
    } catch (err) {
      console.error("Failed to dismiss all:", err);
    }
  };

  const filteredAlerts = showDismissed
    ? alerts
    : alerts.filter((a) => !a.dismissed);

  const dangerCount = filteredAlerts.filter((a) => a.severity === "danger").length;
  const warningCount = filteredAlerts.filter((a) => a.severity === "warning").length;
  const infoCount = filteredAlerts.filter((a) => a.severity === "info").length;

  return (
    <div className="h-full flex flex-col space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Security Alerts</h2>
          <div className="flex gap-3 mt-1">
            {dangerCount > 0 && (
              <span className="text-[10px] text-red-400">
                🚨 {dangerCount} critical
              </span>
            )}
            {warningCount > 0 && (
              <span className="text-[10px] text-amber-400">
                ⚠️ {warningCount} warnings
              </span>
            )}
            {infoCount > 0 && (
              <span className="text-[10px] text-blue-400">
                ℹ️ {infoCount} info
              </span>
            )}
          </div>
        </div>
        <div className="flex gap-2">
          <label className="flex items-center gap-2 text-xs text-gray-400 cursor-pointer">
            <input
              type="checkbox"
              checked={showDismissed}
              onChange={(e) => setShowDismissed(e.target.checked)}
              className="rounded border-gray-600"
            />
            Show dismissed
          </label>
          <button
            onClick={handleDismissAll}
            className="text-xs px-3 py-1.5 rounded-lg bg-white/5 text-gray-400 hover:text-white hover:bg-white/10 transition-colors"
          >
            Dismiss All
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto space-y-2">
        {filteredAlerts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-gray-500">
            <div className="text-4xl mb-3">✅</div>
            <p className="text-sm">No alerts to show</p>
            <p className="text-xs mt-1 text-gray-600">
              Suspicious activity will appear here
            </p>
          </div>
        ) : (
          filteredAlerts.map((alert) => {
            const config = severityConfig[alert.severity] || severityConfig.info;
            return (
              <div
                key={alert.id}
                className={`p-3 rounded-lg border ${config.bg} ${config.border} ${
                  alert.dismissed ? "opacity-50" : ""
                } transition-opacity`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-start gap-3 min-w-0">
                    <span className="text-lg shrink-0 mt-0.5">
                      {config.icon}
                    </span>
                    <div className="min-w-0">
                      <div className="flex items-center gap-2 mb-0.5">
                        <span className={`text-xs font-medium ${config.text}`}>
                          {ruleLabels[alert.rule_name] || alert.rule_name}
                        </span>
                        <span className="text-[10px] text-gray-500">
                          {formatTime(alert.triggered_at)}
                        </span>
                      </div>
                      <p className="text-xs text-gray-400">{alert.message}</p>
                      {alert.process_name && (
                        <p className="text-[10px] text-gray-500 mt-1 font-mono">
                          Process: {alert.process_name}
                        </p>
                      )}
                    </div>
                  </div>
                  {!alert.dismissed && (
                    <button
                      onClick={() => handleDismiss(alert.id)}
                      className="text-xs px-2 py-1 rounded bg-white/5 text-gray-400 hover:text-white hover:bg-white/10 transition-colors shrink-0"
                    >
                      Dismiss
                    </button>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
