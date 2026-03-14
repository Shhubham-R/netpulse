import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DnsQueryRecord } from "../../store/trafficStore";
import AppBadge from "../shared/AppBadge";

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString();
  } catch {
    return iso;
  }
}

export default function DNSLeakView() {
  const [queries, setQueries] = useState<DnsQueryRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterPid, setFilterPid] = useState<string>("");

  const loadQueries = useCallback(async () => {
    setLoading(true);
    try {
      const pid = filterPid ? parseInt(filterPid, 10) : null;
      const result = await invoke<DnsQueryRecord[]>("get_dns_queries", {
        pid: isNaN(pid as number) ? null : pid,
      });
      setQueries(result);
    } catch (err) {
      console.error("Failed to load DNS queries:", err);
    } finally {
      setLoading(false);
    }
  }, [filterPid]);

  useEffect(() => {
    loadQueries();
    const interval = setInterval(loadQueries, 5000);
    return () => clearInterval(interval);
  }, [loadQueries]);

  // Group by process
  const groupedByProcess = queries.reduce<Record<string, DnsQueryRecord[]>>(
    (acc, query) => {
      const key = query.process_name || "Unknown";
      if (!acc[key]) acc[key] = [];
      acc[key].push(query);
      return acc;
    },
    {}
  );

  const resolverStats = queries.reduce<Record<string, number>>((acc, q) => {
    const resolver = q.resolver_ip || "Unknown";
    acc[resolver] = (acc[resolver] || 0) + 1;
    return acc;
  }, {});

  return (
    <div className="h-full flex flex-col space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">DNS Query Log</h2>
          <p className="text-xs text-gray-500 mt-0.5">
            Monitor DNS queries to detect DNS leaks or unexpected resolvers
          </p>
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            placeholder="Filter by PID..."
            value={filterPid}
            onChange={(e) => setFilterPid(e.target.value)}
            className="w-32 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-xs text-white placeholder-gray-500 focus:outline-none focus:border-accent-cyan/50"
            id="dns-filter-pid"
          />
          <button
            onClick={loadQueries}
            className="px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-xs text-gray-300 hover:bg-white/10"
          >
            🔄 Refresh
          </button>
        </div>
      </div>

      {/* Resolver summary */}
      {Object.keys(resolverStats).length > 0 && (
        <div className="glass-panel p-3">
          <h3 className="text-xs font-medium text-gray-400 mb-2">
            DNS Resolvers Used
          </h3>
          <div className="flex flex-wrap gap-2">
            {Object.entries(resolverStats)
              .sort((a, b) => b[1] - a[1])
              .slice(0, 10)
              .map(([resolver, count]) => (
                <div
                  key={resolver}
                  className="px-2 py-1 rounded-lg bg-white/5 border border-white/10 text-[11px]"
                >
                  <span className="font-mono text-accent-cyan">{resolver}</span>
                  <span className="text-gray-500 ml-1.5">({count})</span>
                </div>
              ))}
          </div>
        </div>
      )}

      {/* Query list */}
      <div className="flex-1 overflow-auto">
        {loading ? (
          <div className="flex items-center justify-center h-48">
            <div className="text-gray-500">Loading DNS queries...</div>
          </div>
        ) : queries.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-gray-500">
            <div className="text-3xl mb-3">🔍</div>
            <p className="text-sm">No DNS queries captured</p>
            <p className="text-xs mt-1 text-gray-600">
              DNS queries (UDP port 53) will be logged here
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {Object.entries(groupedByProcess).map(([processName, processQueries]) => (
              <div key={processName} className="glass-panel p-3">
                <div className="flex items-center justify-between mb-2">
                  <AppBadge name={processName} size="sm" />
                  <span className="text-[10px] text-gray-500">
                    {processQueries.length} queries
                  </span>
                </div>
                <div className="space-y-1">
                  {processQueries.slice(0, 20).map((q) => (
                    <div
                      key={q.id}
                      className="flex items-center justify-between py-1 border-b border-white/[0.03] last:border-0"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="badge text-[9px] bg-white/5 border border-white/10 text-gray-400 shrink-0">
                          {q.query_type}
                        </span>
                        <span className="text-xs font-mono text-gray-300 truncate">
                          {q.query_name}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 shrink-0">
                        {q.response_ip && (
                          <span className="text-[10px] font-mono text-accent-emerald">
                            → {q.response_ip}
                          </span>
                        )}
                        {q.resolver_ip && (
                          <span className="text-[10px] font-mono text-gray-500">
                            via {q.resolver_ip}
                          </span>
                        )}
                        <span className="text-[10px] text-gray-600">
                          {formatTime(q.captured_at)}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
