interface CountryFlagProps {
  code: string | null | undefined;
  city?: string | null;
  showLabel?: boolean;
}

function countryCodeToEmoji(code: string): string {
  if (!code || code === "??" || code.length !== 2) return "🌐";
  const offset = 127397;
  return String.fromCodePoint(
    ...code
      .toUpperCase()
      .split("")
      .map((c) => c.charCodeAt(0) + offset)
  );
}

export default function CountryFlag({
  code,
  city,
  showLabel = true,
}: CountryFlagProps) {
  const flag = countryCodeToEmoji(code || "??");

  return (
    <div className="flex items-center gap-1.5 min-w-0">
      <span className="text-sm shrink-0" title={code || "Unknown"}>
        {flag}
      </span>
      {showLabel && (
        <span className="text-xs text-gray-400 truncate">
          {city || code || "—"}
        </span>
      )}
    </div>
  );
}

export { countryCodeToEmoji };
