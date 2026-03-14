interface AppBadgeProps {
  name: string;
  pid?: number | null;
  size?: "sm" | "md";
}

function getAppColor(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 70%, 60%)`;
}

function getAppInitial(name: string): string {
  if (!name || name === "unknown") return "?";
  return name.charAt(0).toUpperCase();
}

export default function AppBadge({ name, pid, size = "md" }: AppBadgeProps) {
  const color = getAppColor(name);
  const initial = getAppInitial(name);
  const isSmall = size === "sm";

  return (
    <div className="flex items-center gap-2 min-w-0">
      <div
        className={`${isSmall ? "w-5 h-5 text-[9px]" : "w-6 h-6 text-[10px]"} rounded-md flex items-center justify-center font-bold shrink-0`}
        style={{ backgroundColor: `${color}22`, color }}
      >
        {initial}
      </div>
      <div className="min-w-0">
        <p
          className={`${isSmall ? "text-[11px]" : "text-xs"} font-medium text-white truncate`}
        >
          {name}
        </p>
        {pid && !isSmall && (
          <p className="text-[10px] text-gray-500 font-mono">PID {pid}</p>
        )}
      </div>
    </div>
  );
}

export { getAppColor };
