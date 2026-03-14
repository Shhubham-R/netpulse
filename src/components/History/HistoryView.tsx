import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ConnectionRecord } from "../../store/trafficStore";
import AppBadge from "../shared/AppBadge";
import CountryFlag from "../shared/CountryFlag";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

interface HistoryFilter {
  process_name: string | null;
  country_code: string | null;
  hostname: string | null;
  start_date: string | null;
  end_date: string | null;
  limit: number | null;
  offset: number | null;
}

export default function HistoryView() {
  const [connections, setConnections] = useState<ConnectionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");
  const [countryFilter, setCountryFilter] = useState("");
  const [page, setPage] = useState(0);
  const pageSize = 100;

  const loadHistory = useCallback(async () => {
    setLoading(true);
    try {
      const filter: HistoryFilter = {
        process_name: searchTerm || null,
        country_code: countryFilter || null,
        hostname: null,
        start_date: null,
        end_date: null,
        limit: pageSize,
        offset: page * pageSize,
      };
      const result = await invoke<ConnectionRecord[]>("get_connection_history", {
        filter,
      });
      setConnections(result);
    } catch (err) {
      console.error("Failed to load history:", err);
    } finally {
      setLoading(false);
    }
  }, [searchTerm, countryFilter, page]);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const handleExport = async (format: string) => {
    try {
      const path = await invoke<string>("export_session", { format });
      alert(`Exported to: ${path}`);
    } catch (err) {
      console.error("Export failed:", err);
    }
  };

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Connection History</h2>
        <div className="flex gap-2">
          <button
            onClick={() => handleExport("csv")}
            className="btn-primary text-xs"
            id="export-csv"
          >
            Export CSV
          </button>
          <button
            onClick={() => handleExport("json")}
            className="btn-primary text-xs"
            id="export-json"
          >
            Export JSON
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-3">
        <input
          type="text"
          placeholder="Search by app name..."
          value={searchTerm}
          onChange={(e) => {
            setSearchTerm(e.target.value);
            setPage(0);
          }}
          className="flex-1 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-accent-cyan/50 transition-colors"
          id="history-search"
        />
        <input
          type="text"
          placeholder="Country (e.g. US)"
          value={countryFilter}
          onChange={(e) => {
            setCountryFilter(e.target.value.toUpperCase());
            setPage(0);
          }}
          className="w-32 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-accent-cyan/50 transition-colors"
          id="history-country"
        />
        <button
          onClick={loadHistory}
          className="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-gray-300 hover:bg-white/10 transition-colors"
        >
          🔄
        </button>
      </div>

      {/* Table */}
      <div className="flex-1 glass-panel overflow-auto">
        {loading ? (
          <div className="flex items-center justify-center h-48">
            <div className="text-gray-500">Loading...</div>
          </div>
        ) : connections.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-gray-500">
            <div className="text-3xl mb-3">📭</div>
            <p className="text-sm">No connection history found</p>
          </div>
        ) : (
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-panel-border text-[10px] uppercase text-gray-500">
                <th className="text-left px-3 py-2 font-medium">App</th>
                <th className="text-left px-3 py-2 font-medium">Protocol</th>
                <th className="text-left px-3 py-2 font-medium">Remote Host</th>
                <th className="text-left px-3 py-2 font-medium">Country</th>
                <th className="text-right px-3 py-2 font-medium">↑ Sent</th>
                <th className="text-right px-3 py-2 font-medium">↓ Recv</th>
                <th className="text-left px-3 py-2 font-medium">First Seen</th>
                <th className="text-left px-3 py-2 font-medium">Last Seen</th>
              </tr>
            </thead>
            <tbody>
              {connections.map((conn) => (
                <tr key={conn.id} className="table-row">
                  <td className="px-3 py-2">
                    <AppBadge name={conn.process_name} pid={conn.pid} size="sm" />
                  </td>
                  <td className="px-3 py-2 text-gray-400">{conn.protocol}</td>
                  <td className="px-3 py-2">
                    <span className="font-mono text-gray-300 truncate block max-w-[200px]">
                      {conn.dst_host || conn.dst_ip}
                    </span>
                  </td>
                  <td className="px-3 py-2">
                    <CountryFlag code={conn.country_code} city={conn.city} />
                  </td>
                  <td className="px-3 py-2 text-right font-mono text-accent-cyan">
                    {formatBytes(conn.bytes_sent)}
                  </td>
                  <td className="px-3 py-2 text-right font-mono text-accent-emerald">
                    {formatBytes(conn.bytes_recv)}
                  </td>
                  <td className="px-3 py-2 text-gray-400">
                    {formatDate(conn.first_seen)}
                  </td>
                  <td className="px-3 py-2 text-gray-400">
                    {formatDate(conn.last_seen)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between shrink-0">
        <span className="text-xs text-gray-500">
          Page {page + 1} · {connections.length} results
        </span>
        <div className="flex gap-2">
          <button
            onClick={() => setPage(Math.max(0, page - 1))}
            disabled={page === 0}
            className="px-3 py-1.5 rounded text-xs bg-white/5 text-gray-300 disabled:opacity-30 hover:bg-white/10 transition-colors"
          >
            ← Prev
          </button>
          <button
            onClick={() => setPage(page + 1)}
            disabled={connections.length < pageSize}
            className="px-3 py-1.5 rounded text-xs bg-white/5 text-gray-300 disabled:opacity-30 hover:bg-white/10 transition-colors"
          >
            Next →
          </button>
        </div>
      </div>
    </div>
  );
}
