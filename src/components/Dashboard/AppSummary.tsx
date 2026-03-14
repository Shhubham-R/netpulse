import { useMemo } from "react";
import { useTrafficStore } from "../../store/trafficStore";
import AppBadge from "../shared/AppBadge";
import CountryFlag from "../shared/CountryFlag";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function AppSummary() {
  const appSummaries = useTrafficStore((s) => s.appSummaries);

  const sortedApps = useMemo(() => {
    return Array.from(appSummaries.values())
      .sort(
        (a, b) =>
          b.total_bytes_sent +
          b.total_bytes_recv -
          (a.total_bytes_sent + a.total_bytes_recv)
      )
      .slice(0, 20);
  }, [appSummaries]);

  if (sortedApps.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 text-gray-500 text-sm">
        <div className="text-center">
          <div className="text-2xl mb-2">📱</div>
          <p>No app data yet</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {sortedApps.map((app) => {
        const countries = Array.from(app.countries).slice(0, 3);
        const hostnames = Array.from(app.hostnames).slice(0, 2);

        return (
          <div
            key={app.process_name}
            className="p-2.5 rounded-lg bg-white/[0.02] hover:bg-white/[0.04] transition-colors border border-white/[0.04]"
          >
            <div className="flex items-center justify-between mb-1.5">
              <AppBadge name={app.process_name} size="sm" />
              <span className="text-[10px] text-gray-500">
                {app.connection_count} conn
              </span>
            </div>

            {hostnames.length > 0 && (
              <p className="text-[11px] text-gray-400 truncate mb-1">
                → {hostnames.join(", ")}
              </p>
            )}

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {countries.map((code) => (
                  <CountryFlag key={code} code={code} showLabel={false} />
                ))}
              </div>
              <div className="flex items-center gap-3 text-[10px] font-mono">
                <span className="text-accent-cyan">
                  ↑ {formatBytes(app.total_bytes_sent)}
                </span>
                <span className="text-accent-emerald">
                  ↓ {formatBytes(app.total_bytes_recv)}
                </span>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
