import { useMemo, useState } from "react";
import { useTrafficStore } from "../../store/trafficStore";
import { countryCodeToEmoji } from "../shared/CountryFlag";

// Simplified world map SVG paths (Natural Earth projection, simplified for inline use)
// Each entry: [country_code, path_d, cx, cy] where cx,cy is approximate centroid
const COUNTRY_PATHS: Array<[string, number, number]> = [
  ["US", -98, 38],
  ["CA", -106, 56],
  ["MX", -102, 23],
  ["BR", -51, -14],
  ["AR", -64, -34],
  ["GB", -2, 54],
  ["FR", 2, 47],
  ["DE", 10, 51],
  ["IT", 12, 42],
  ["ES", -4, 40],
  ["PT", -8, 39],
  ["NL", 5, 52],
  ["BE", 4, 51],
  ["SE", 15, 62],
  ["NO", 8, 62],
  ["FI", 26, 64],
  ["DK", 10, 56],
  ["PL", 20, 52],
  ["CZ", 15, 50],
  ["AT", 14, 47],
  ["CH", 8, 47],
  ["RU", 105, 60],
  ["UA", 32, 49],
  ["TR", 35, 39],
  ["IL", 35, 31],
  ["AE", 54, 24],
  ["SA", 45, 25],
  ["IN", 79, 21],
  ["CN", 104, 35],
  ["JP", 138, 36],
  ["KR", 128, 36],
  ["TW", 121, 24],
  ["SG", 104, 1],
  ["AU", 134, -26],
  ["NZ", 174, -41],
  ["ZA", 25, -29],
  ["NG", 8, 10],
  ["EG", 30, 27],
  ["KE", 38, 0],
];

function lonToX(lon: number): number {
  return ((lon + 180) / 360) * 800;
}

function latToY(lat: number): number {
  // Mercator-like projection
  const latRad = (lat * Math.PI) / 180;
  const mercN = Math.log(Math.tan(Math.PI / 4 + latRad / 2));
  return 250 - (mercN / Math.PI) * 250;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

interface CountryData {
  code: string;
  connections: number;
  totalBytes: number;
  apps: Set<string>;
  x: number;
  y: number;
}

export default function GeoMap() {
  const liveConnections = useTrafficStore((s) => s.liveConnections);
  const [hoveredCountry, setHoveredCountry] = useState<CountryData | null>(null);
  const [tooltipPos, setTooltipPos] = useState({ x: 0, y: 0 });

  const countryData = useMemo(() => {
    const map = new Map<string, CountryData>();

    for (const conn of liveConnections.values()) {
      const code = conn.country_code;
      if (!code || code === "??") continue;

      const existing = map.get(code);
      if (existing) {
        existing.connections++;
        existing.totalBytes += conn.payload_len;
        existing.apps.add(conn.process_name);
      } else {
        const countryInfo = COUNTRY_PATHS.find(([c]) => c === code);
        const lon = countryInfo ? countryInfo[1] : 0;
        const lat = countryInfo ? countryInfo[2] : 0;

        map.set(code, {
          code,
          connections: 1,
          totalBytes: conn.payload_len,
          apps: new Set([conn.process_name]),
          x: lonToX(lon),
          y: latToY(lat),
        });
      }
    }

    return Array.from(map.values());
  }, [liveConnections]);

  const maxBytes = Math.max(1, ...countryData.map((d) => d.totalBytes));

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold">Connection World Map</h2>
        <span className="text-xs text-gray-500">
          {countryData.length} countries · {liveConnections.size} connections
        </span>
      </div>

      <div className="flex-1 glass-panel p-4 relative overflow-hidden">
        <svg
          viewBox="0 0 800 500"
          className="w-full h-full"
          style={{ maxHeight: "calc(100vh - 200px)" }}
        >
          {/* Background grid */}
          <defs>
            <radialGradient id="dot-glow" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="#00d4ff" stopOpacity="0.8" />
              <stop offset="100%" stopColor="#00d4ff" stopOpacity="0" />
            </radialGradient>
            <radialGradient id="dot-glow-amber" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="#ffb800" stopOpacity="0.8" />
              <stop offset="100%" stopColor="#ffb800" stopOpacity="0" />
            </radialGradient>
            <radialGradient id="dot-glow-red" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="#ff4444" stopOpacity="0.8" />
              <stop offset="100%" stopColor="#ff4444" stopOpacity="0" />
            </radialGradient>
          </defs>

          {/* Grid lines */}
          {Array.from({ length: 19 }, (_, i) => (
            <line
              key={`lon-${i}`}
              x1={i * (800 / 18)}
              y1={0}
              x2={i * (800 / 18)}
              y2={500}
              stroke="rgba(255,255,255,0.02)"
              strokeWidth="0.5"
            />
          ))}
          {Array.from({ length: 10 }, (_, i) => (
            <line
              key={`lat-${i}`}
              x1={0}
              y1={i * 50}
              x2={800}
              y2={i * 50}
              stroke="rgba(255,255,255,0.02)"
              strokeWidth="0.5"
            />
          ))}

          {/* Country reference dots (faded) */}
          {COUNTRY_PATHS.map(([code, lon, lat]) => {
            const hasTraffic = countryData.some((d) => d.code === code);
            if (hasTraffic) return null;
            return (
              <circle
                key={code}
                cx={lonToX(lon)}
                cy={latToY(lat)}
                r={2}
                fill="rgba(255,255,255,0.08)"
              />
            );
          })}

          {/* Active connection dots */}
          {countryData.map((country) => {
            const radius = 4 + (country.totalBytes / maxBytes) * 16;
            return (
              <g key={country.code}>
                {/* Glow */}
                <circle
                  cx={country.x}
                  cy={country.y}
                  r={radius * 2}
                  fill="url(#dot-glow)"
                  opacity={0.3}
                  className="animate-pulse-slow"
                />
                {/* Dot */}
                <circle
                  cx={country.x}
                  cy={country.y}
                  r={radius}
                  fill="#00d4ff"
                  opacity={0.8}
                  stroke="#00d4ff"
                  strokeWidth={1}
                  className="cursor-pointer"
                  onMouseEnter={(e) => {
                    setHoveredCountry(country);
                    const rect = (
                      e.target as SVGElement
                    ).closest("svg")?.getBoundingClientRect();
                    if (rect) {
                      setTooltipPos({
                        x: e.clientX - rect.left,
                        y: e.clientY - rect.top,
                      });
                    }
                  }}
                  onMouseLeave={() => setHoveredCountry(null)}
                />
                {/* Label */}
                {radius > 8 && (
                  <text
                    x={country.x}
                    y={country.y - radius - 4}
                    textAnchor="middle"
                    fill="rgba(255,255,255,0.6)"
                    fontSize="9"
                    fontFamily="Inter"
                  >
                    {country.code}
                  </text>
                )}
              </g>
            );
          })}
        </svg>

        {/* Tooltip */}
        {hoveredCountry && (
          <div
            className="absolute z-10 glass-panel p-3 rounded-lg shadow-xl pointer-events-none animate-fade-in"
            style={{
              left: Math.min(tooltipPos.x + 10, 600),
              top: Math.max(tooltipPos.y - 80, 10),
              minWidth: "180px",
            }}
          >
            <div className="flex items-center gap-2 mb-2">
              <span className="text-lg">
                {countryCodeToEmoji(hoveredCountry.code)}
              </span>
              <span className="text-sm font-medium">{hoveredCountry.code}</span>
            </div>
            <div className="space-y-1 text-[11px]">
              <div className="flex justify-between">
                <span className="text-gray-500">Connections:</span>
                <span className="text-white">{hoveredCountry.connections}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Traffic:</span>
                <span className="text-accent-cyan">
                  {formatBytes(hoveredCountry.totalBytes)}
                </span>
              </div>
              <div className="mt-1.5 pt-1.5 border-t border-white/10">
                <span className="text-gray-500">Apps:</span>
                <p className="text-[10px] text-gray-300 mt-0.5">
                  {Array.from(hoveredCountry.apps).slice(0, 5).join(", ")}
                </p>
              </div>
            </div>
          </div>
        )}

        {countryData.length === 0 && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center text-gray-500">
              <div className="text-4xl mb-3 opacity-30">🌍</div>
              <p className="text-sm">No geographic data available</p>
              <p className="text-xs mt-1 text-gray-600">
                GeoIP data will appear when connections are detected
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
