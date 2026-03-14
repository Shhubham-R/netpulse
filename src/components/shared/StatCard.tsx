interface StatCardProps {
  label: string;
  value: string;
  color: "cyan" | "emerald" | "amber" | "red" | "white";
}

const colorMap = {
  cyan: "text-accent-cyan",
  emerald: "text-accent-emerald",
  amber: "text-accent-amber",
  red: "text-accent-red",
  white: "text-white",
};

export default function StatCard({ label, value, color }: StatCardProps) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-[10px] text-gray-500 uppercase tracking-wider">
        {label}
      </span>
      <span className={`font-mono text-xs font-medium ${colorMap[color]}`}>
        {value}
      </span>
    </div>
  );
}
