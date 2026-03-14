import { useMemo, useCallback, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { invoke } from "@tauri-apps/api/core";
import { useTrafficStore, ConnectionEvent } from "../../store/trafficStore";
import AppBadge from "../shared/AppBadge";
import CountryFlag from "../shared/CountryFlag";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

type SortKey = "process_name" | "protocol" | "dst_ip" | "country_code" | "payload_len" | "timestamp";
type SortDir = "asc" | "desc";

export default function LiveTable() {
  const liveConnections = useTrafficStore((s) => s.liveConnections);
  const [sortKey, setSortKey] = useState<SortKey>("timestamp");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  const connections = useMemo(() => {
    const arr = Array.from(liveConnections.values());
    arr.sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case "process_name":
          cmp = a.process_name.localeCompare(b.process_name);
          break;
        case "protocol":
          cmp = a.protocol.localeCompare(b.protocol);
          break;
        case "dst_ip":
          cmp = a.dst_ip.localeCompare(b.dst_ip);
          break;
        case "country_code":
          cmp = a.country_code.localeCompare(b.country_code);
          break;
        case "payload_len":
          cmp = a.payload_len - b.payload_len;
          break;
        case "timestamp":
          cmp = a.timestamp.localeCompare(b.timestamp);
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return arr;
  }, [liveConnections, sortKey, sortDir]);

  const parentRef = useCallback((node: HTMLDivElement | null) => {
    if (node) parentNode.current = node;
  }, []);
  const parentNode = { current: null as HTMLDivElement | null };

  const virtualizer = useVirtualizer({
    count: connections.length,
    getScrollElement: () => parentNode.current,
    estimateSize: () => 40,
    overscan: 20,
  });

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const handleBlock = async (processName: string) => {
    try {
      await invoke("block_app", { processName });
    } catch (err) {
      console.error("Failed to block app:", err);
    }
  };

  const handleTrust = async (processName: string) => {
    try {
      await invoke("trust_app", { processName });
    } catch (err) {
      console.error("Failed to trust app:", err);
    }
  };

  const SortHeader = ({ label, field }: { label: string; field: SortKey }) => (
    <button
      onClick={() => handleSort(field)}
      className="flex items-center gap-1 text-[10px] uppercase tracking-wider text-gray-500 hover:text-gray-300 transition-colors font-medium"
    >
      {label}
      {sortKey === field && (
        <span className="text-accent-cyan">
          {sortDir === "asc" ? "↑" : "↓"}
        </span>
      )}
    </button>
  );

  if (connections.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-48 text-gray-500">
        <div className="text-3xl mb-3">📡</div>
        <p className="text-sm">Waiting for network traffic...</p>
        <p className="text-xs mt-1 text-gray-600">
          Make sure packet capture is running with the right permissions
        </p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="grid grid-cols-12 gap-2 px-3 py-2 border-b border-panel-border shrink-0">
        <div className="col-span-2">
          <SortHeader label="Application" field="process_name" />
        </div>
        <div className="col-span-1">
          <SortHeader label="Proto" field="protocol" />
        </div>
        <div className="col-span-2">
          <SortHeader label="Remote Host" field="dst_ip" />
        </div>
        <div className="col-span-1">
          <SortHeader label="Country" field="country_code" />
        </div>
        <div className="col-span-1">
          <SortHeader label="↑ Sent" field="payload_len" />
        </div>
        <div className="col-span-1">
          <span className="text-[10px] uppercase tracking-wider text-gray-500">
            ↓ Recv
          </span>
        </div>
        <div className="col-span-1">
          <span className="text-[10px] uppercase tracking-wider text-gray-500">
            Port
          </span>
        </div>
        <div className="col-span-1">
          <span className="text-[10px] uppercase tracking-wider text-gray-500">
            Status
          </span>
        </div>
        <div className="col-span-2">
          <span className="text-[10px] uppercase tracking-wider text-gray-500">
            Actions
          </span>
        </div>
      </div>

      {/* Virtual scrolled body */}
      <div
        ref={parentRef}
        className="flex-1 overflow-auto"
        style={{ contain: "strict" }}
      >
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: "100%",
            position: "relative",
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const conn = connections[virtualRow.index];
            const key = `${conn.process_name}-${conn.dst_ip}-${conn.dst_port}`;
            const isExpanded = expandedRow === key;

            return (
              <div
                key={key}
                className="table-row cursor-pointer"
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                onClick={() => setExpandedRow(isExpanded ? null : key)}
              >
                <div className="grid grid-cols-12 gap-2 px-3 py-2 items-center h-full">
                  <div className="col-span-2">
                    <AppBadge name={conn.process_name} pid={conn.pid} size="sm" />
                  </div>
                  <div className="col-span-1">
                    <span className="badge text-[10px] bg-white/5 text-gray-400 border border-white/10">
                      {conn.protocol}
                    </span>
                  </div>
                  <div className="col-span-2">
                    <p className="text-xs font-mono text-gray-300 truncate" title={conn.dst_ip}>
                      {conn.hostname || conn.dst_ip}
                    </p>
                  </div>
                  <div className="col-span-1">
                    <CountryFlag code={conn.country_code} city={conn.city} showLabel={false} />
                  </div>
                  <div className="col-span-1">
                    <span className="text-xs font-mono text-accent-cyan">
                      {conn.is_outbound ? formatBytes(conn.payload_len) : "—"}
                    </span>
                  </div>
                  <div className="col-span-1">
                    <span className="text-xs font-mono text-accent-emerald">
                      {!conn.is_outbound ? formatBytes(conn.payload_len) : "—"}
                    </span>
                  </div>
                  <div className="col-span-1">
                    <span className="text-xs font-mono text-gray-400">
                      {conn.dst_port}
                    </span>
                  </div>
                  <div className="col-span-1">
                    <span className="badge-active text-[10px]">ACTIVE</span>
                  </div>
                  <div className="col-span-2 flex gap-1">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleBlock(conn.process_name);
                      }}
                      className="text-[10px] px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors"
                      title="Block"
                    >
                      Block
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleTrust(conn.process_name);
                      }}
                      className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 transition-colors"
                      title="Trust"
                    >
                      Trust
                    </button>
                  </div>
                </div>

                {/* Expanded row details */}
                {isExpanded && (
                  <div className="px-3 py-2 bg-white/[0.02] border-t border-panel-border animate-fade-in">
                    <div className="grid grid-cols-4 gap-4 text-[11px]">
                      <div>
                        <span className="text-gray-500">Source IP:</span>{" "}
                        <span className="font-mono text-gray-300">{conn.src_ip}</span>
                      </div>
                      <div>
                        <span className="text-gray-500">Source Port:</span>{" "}
                        <span className="font-mono text-gray-300">{conn.src_port}</span>
                      </div>
                      <div>
                        <span className="text-gray-500">Country:</span>{" "}
                        <span className="text-gray-300">
                          {conn.country_name} ({conn.country_code})
                        </span>
                      </div>
                      <div>
                        <span className="text-gray-500">City:</span>{" "}
                        <span className="text-gray-300">{conn.city || "—"}</span>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      <div className="shrink-0 px-3 py-1.5 border-t border-panel-border text-[10px] text-gray-500">
        {connections.length} active connections
      </div>
    </div>
  );
}
