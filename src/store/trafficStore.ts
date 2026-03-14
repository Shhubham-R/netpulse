import { create } from "zustand";

export interface ConnectionEvent {
  src_ip: string;
  src_port: number;
  dst_ip: string;
  dst_port: number;
  protocol: string;
  payload_len: number;
  timestamp: string;
  is_outbound: boolean;
  process_name: string;
  pid: number | null;
  country_code: string;
  country_name: string;
  city: string;
  latitude: number;
  longitude: number;
  hostname: string | null;
}

export interface ConnectionRecord {
  id: number;
  pid: number | null;
  process_name: string;
  protocol: string;
  src_port: number;
  dst_ip: string;
  dst_host: string | null;
  country_code: string | null;
  city: string | null;
  bytes_sent: number;
  bytes_recv: number;
  first_seen: string;
  last_seen: string;
  is_blocked: boolean;
  is_trusted: boolean;
}

export interface AlertRecord {
  id: number;
  connection_id: number | null;
  rule_name: string;
  severity: string;
  message: string;
  process_name: string | null;
  triggered_at: string;
  dismissed: boolean;
}

export interface DnsQueryRecord {
  id: number;
  pid: number | null;
  process_name: string | null;
  query_name: string;
  query_type: string;
  resolver_ip: string | null;
  response_ip: string | null;
  captured_at: string;
}

export interface BandwidthPoint {
  timestamp: string;
  app_name: string;
  upload_kbps: number;
  download_kbps: number;
}

export interface SpeedStats {
  total_upload: number;
  total_download: number;
  upload_speed: number;
  download_speed: number;
}

export interface AppSummaryData {
  process_name: string;
  total_bytes_sent: number;
  total_bytes_recv: number;
  connection_count: number;
  countries: Set<string>;
  hostnames: Set<string>;
  last_seen: string;
}

export type ViewType = "dashboard" | "history" | "map" | "alerts" | "dns" | "settings";

interface TrafficState {
  // Live connections from events
  liveConnections: Map<string, ConnectionEvent>;
  // Active connection records from DB
  activeConnections: ConnectionRecord[];
  // Connection history from DB
  connectionHistory: ConnectionRecord[];
  // Bandwidth data points
  bandwidthSeries: BandwidthPoint[];
  // Alerts
  alerts: AlertRecord[];
  // DNS queries
  dnsQueries: DnsQueryRecord[];
  // Speed stats
  speedStats: SpeedStats;
  // App summaries
  appSummaries: Map<string, AppSummaryData>;
  // Current view
  currentView: ViewType;
  // Theme
  theme: "dark" | "light";
  // Capture status
  captureRunning: boolean;
  // Loading states
  loadingHistory: boolean;
  loadingAlerts: boolean;
  loadingDns: boolean;

  // Actions
  addTrafficEvents: (events: ConnectionEvent[]) => void;
  setActiveConnections: (connections: ConnectionRecord[]) => void;
  setConnectionHistory: (history: ConnectionRecord[]) => void;
  setBandwidthSeries: (series: BandwidthPoint[]) => void;
  setAlerts: (alerts: AlertRecord[]) => void;
  addAlert: (alert: AlertRecord) => void;
  setDnsQueries: (queries: DnsQueryRecord[]) => void;
  updateSpeedStats: (stats: SpeedStats) => void;
  setCurrentView: (view: ViewType) => void;
  setTheme: (theme: "dark" | "light") => void;
  setCaptureRunning: (running: boolean) => void;
  setLoadingHistory: (loading: boolean) => void;
  setLoadingAlerts: (loading: boolean) => void;
  setLoadingDns: (loading: boolean) => void;
}

export const useTrafficStore = create<TrafficState>((set, get) => ({
  liveConnections: new Map(),
  activeConnections: [],
  connectionHistory: [],
  bandwidthSeries: [],
  alerts: [],
  dnsQueries: [],
  speedStats: {
    total_upload: 0,
    total_download: 0,
    upload_speed: 0,
    download_speed: 0,
  },
  appSummaries: new Map(),
  currentView: "dashboard",
  theme: "dark",
  captureRunning: false,
  loadingHistory: false,
  loadingAlerts: false,
  loadingDns: false,

  addTrafficEvents: (events) => {
    set((state) => {
      const newConnections = new Map(state.liveConnections);
      const newSummaries = new Map(state.appSummaries);

      for (const event of events) {
        const key = `${event.process_name}-${event.dst_ip}-${event.dst_port}`;
        newConnections.set(key, event);

        // Update app summaries
        const existing = newSummaries.get(event.process_name) || {
          process_name: event.process_name,
          total_bytes_sent: 0,
          total_bytes_recv: 0,
          connection_count: 0,
          countries: new Set<string>(),
          hostnames: new Set<string>(),
          last_seen: event.timestamp,
        };

        if (event.is_outbound) {
          existing.total_bytes_sent += event.payload_len;
        } else {
          existing.total_bytes_recv += event.payload_len;
        }

        existing.connection_count++;
        if (event.country_code) existing.countries.add(event.country_code);
        if (event.hostname) existing.hostnames.add(event.hostname);
        existing.last_seen = event.timestamp;

        newSummaries.set(event.process_name, existing);
      }

      // Trim old connections (keep last 30 seconds)
      const cutoff = new Date(Date.now() - 30000).toISOString();
      for (const [key, conn] of newConnections) {
        if (conn.timestamp < cutoff) {
          newConnections.delete(key);
        }
      }

      return {
        liveConnections: newConnections,
        appSummaries: newSummaries,
        captureRunning: true,
      };
    });
  },

  setActiveConnections: (connections) => set({ activeConnections: connections }),
  setConnectionHistory: (history) => set({ connectionHistory: history, loadingHistory: false }),
  setBandwidthSeries: (series) => set({ bandwidthSeries: series }),
  setAlerts: (alerts) => set({ alerts, loadingAlerts: false }),

  addAlert: (alert) => {
    set((state) => ({
      alerts: [alert, ...state.alerts].slice(0, 200),
    }));
  },

  setDnsQueries: (queries) => set({ dnsQueries: queries, loadingDns: false }),
  updateSpeedStats: (stats) => set({ speedStats: stats }),
  setCurrentView: (view) => set({ currentView: view }),

  setTheme: (theme) => {
    document.documentElement.classList.toggle("light", theme === "light");
    document.documentElement.classList.toggle("dark", theme === "dark");
    set({ theme });
  },

  setCaptureRunning: (running) => set({ captureRunning: running }),
  setLoadingHistory: (loading) => set({ loadingHistory: loading }),
  setLoadingAlerts: (loading) => set({ loadingAlerts: loading }),
  setLoadingDns: (loading) => set({ loadingDns: loading }),
}));
