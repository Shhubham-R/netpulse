import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTrafficStore, AlertRecord } from "../store/trafficStore";

export function useAlerts() {
  const addAlert = useTrafficStore((s) => s.addAlert);
  const isListening = useRef(false);

  useEffect(() => {
    if (isListening.current) return;
    isListening.current = true;

    let unlisten: (() => void) | null = null;

    const setup = async () => {
      unlisten = await listen<AlertRecord>("alert-triggered", (event) => {
        addAlert(event.payload);

        // Show toast notification
        showToast(event.payload);
      });
    };

    setup();

    return () => {
      unlisten?.();
      isListening.current = false;
    };
  }, [addAlert]);
}

function showToast(alert: AlertRecord) {
  // Create toast element
  const toast = document.createElement("div");
  toast.className = `fixed bottom-4 right-4 z-50 max-w-sm p-4 rounded-lg shadow-xl animate-slide-in ${
    alert.severity === "danger"
      ? "bg-red-900/90 border border-red-500/50"
      : alert.severity === "warning"
        ? "bg-amber-900/90 border border-amber-500/50"
        : "bg-blue-900/90 border border-blue-500/50"
  }`;

  const icon = alert.severity === "danger" ? "🚨" : alert.severity === "warning" ? "⚠️" : "ℹ️";

  toast.innerHTML = `
    <div class="flex items-start gap-3">
      <span class="text-lg">${icon}</span>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-white">${alert.rule_name.replace(/_/g, " ").replace(/\b\w/g, (l) => l.toUpperCase())}</p>
        <p class="text-xs text-gray-300 mt-1 truncate">${alert.message}</p>
      </div>
    </div>
  `;

  document.body.appendChild(toast);

  setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transition = "opacity 0.3s ease-out";
    setTimeout(() => toast.remove(), 300);
  }, 5000);
}
