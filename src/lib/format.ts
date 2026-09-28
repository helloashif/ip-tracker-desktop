export function fmtDateTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleString(undefined, {
    year: "numeric", month: "short", day: "2-digit", hour: "2-digit", minute: "2-digit",
  });
}

export function fmtDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "2-digit" });
}

export function fmtTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

export function fmtDuration(seconds: number | null | undefined): string {
  if (seconds == null || !isFinite(seconds)) return "—";
  const s = Math.max(0, Math.round(seconds));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${s}s`;
}

export function sinceNow(iso: string | null | undefined): string {
  if (!iso) return "—";
  return fmtDuration((Date.now() - new Date(iso).getTime()) / 1000);
}

export function untilNow(iso: string | null | undefined): string {
  if (!iso) return "—";
  const secs = (new Date(iso).getTime() - Date.now()) / 1000;
  return secs <= 0 ? "now" : `in ${fmtDuration(secs)}`;
}

export function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function kindLabel(kind: string): string {
  switch (kind) {
    case "start": return "Tracking started";
    case "change": return "Address changed";
    case "offline": return "Went offline";
    case "online": return "Back online";
    default: return kind;
  }
}

export function kindClass(kind: string): string {
  switch (kind) {
    case "change": return "bg-warn/15 text-warn";
    case "offline": return "bg-danger/15 text-danger";
    case "online": return "bg-ok/15 text-ok";
    default: return "bg-surface2 text-fg2";
  }
}
