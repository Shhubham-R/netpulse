import { useMemo, useState } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { useTrafficStore } from "../../store/trafficStore";
import { getAppColor } from "../shared/AppBadge";

interface ChartDataPoint {
  time: string;
  [appName: string]: number | string;
}

export default function BandwidthGraph() {
  const liveConnections = useTrafficStore((s) => s.liveConnections);
  const [hiddenApps, setHiddenApps] = useState<Set<string>>(new Set());

  // Build time-series data from live connections
  const { chartData, appNames } = useMemo(() => {
    const appMap = new Map<string, { upload: number; download: number }>();

    for (const conn of liveConnections.values()) {
      const existing = appMap.get(conn.process_name) || { upload: 0, download: 0 };
      if (conn.is_outbound) {
        existing.upload += conn.payload_len;
      } else {
        existing.download += conn.payload_len;
      }
      appMap.set(conn.process_name, existing);
    }

    const names = Array.from(appMap.keys()).slice(0, 10); // Limit to top 10 apps
    const now = new Date();
    const data: ChartDataPoint[] = [];

    // Generate 60 data points (one per second for last 60 seconds)
    for (let i = 59; i >= 0; i--) {
      const t = new Date(now.getTime() - i * 1000);
      const point: ChartDataPoint = {
        time: t.toLocaleTimeString("en-US", {
          hour12: false,
          minute: "2-digit",
          second: "2-digit",
        }),
      };

      for (const name of names) {
        const stats = appMap.get(name);
        if (stats) {
          // Simulate distribution across time (in a real app, we'd track per-second)
          const jitter = 0.5 + Math.random() * 1.0;
          point[`${name}_up`] = i < 5 ? (stats.upload / 60) * jitter / 1024 : 0;
          point[`${name}_down`] = i < 5 ? (stats.download / 60) * jitter / 1024 : 0;
        }
      }

      data.push(point);
    }

    return { chartData: data, appNames: names };
  }, [liveConnections]);

  const toggleApp = (name: string) => {
    setHiddenApps((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  };

  if (appNames.length === 0) {
    return (
      <div className="flex items-center justify-center h-44 text-gray-500 text-sm">
        <div className="text-center">
          <div className="text-2xl mb-2">📈</div>
          <p>No bandwidth data yet</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-48">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData} margin={{ top: 5, right: 5, bottom: 5, left: 0 }}>
          <defs>
            {appNames.map((name) => (
              <linearGradient key={name} id={`gradient-${name}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={getAppColor(name)} stopOpacity={0.3} />
                <stop offset="95%" stopColor={getAppColor(name)} stopOpacity={0} />
              </linearGradient>
            ))}
          </defs>
          <CartesianGrid
            strokeDasharray="3 3"
            stroke="rgba(255,255,255,0.04)"
            vertical={false}
          />
          <XAxis
            dataKey="time"
            tick={{ fontSize: 9, fill: "#64748b" }}
            axisLine={{ stroke: "rgba(255,255,255,0.06)" }}
            tickLine={false}
            interval="preserveStartEnd"
          />
          <YAxis
            tick={{ fontSize: 9, fill: "#64748b" }}
            axisLine={{ stroke: "rgba(255,255,255,0.06)" }}
            tickLine={false}
            tickFormatter={(v: number) => `${v.toFixed(0)}KB`}
            width={45}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "rgba(10, 14, 26, 0.95)",
              border: "1px solid rgba(255,255,255,0.1)",
              borderRadius: "8px",
              fontSize: "11px",
              color: "#e2e8f0",
            }}
            formatter={(value: number, name: string) => [
              `${value.toFixed(1)} KB/s`,
              name.replace(/_up$/, " ↑").replace(/_down$/, " ↓"),
            ]}
          />
          <Legend
            wrapperStyle={{ fontSize: "10px", cursor: "pointer" }}
            onClick={(entry) => {
              if (entry.dataKey) {
                const appName = String(entry.dataKey).replace(/_up$|_down$/, "");
                toggleApp(appName);
              }
            }}
            formatter={(value: string) =>
              value.replace(/_up$/, " ↑").replace(/_down$/, " ↓")
            }
          />
          {appNames
            .filter((name) => !hiddenApps.has(name))
            .map((name) => (
              <Area
                key={`${name}_up`}
                type="monotone"
                dataKey={`${name}_up`}
                stackId="upload"
                stroke={getAppColor(name)}
                fill={`url(#gradient-${name})`}
                strokeWidth={1.5}
                dot={false}
                animationDuration={300}
              />
            ))}
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
