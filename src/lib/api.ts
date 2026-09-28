import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type EventKind = "start" | "change" | "offline" | "online";

export interface IpEvent {
  id: number;
  ts: string;
  kind: EventKind;
  public_ip: string | null;
  prev_public_ip: string | null;
  public_ipv6: string | null;
  prev_public_ipv6: string | null;
  local_ips: string[];
  isp: string | null;
  country: string | null;
  city: string | null;
  label: string | null;
}

export interface CurrentStatus {
  public_ip: string | null;
  public_ipv6: string | null;
  local_ips: string[];
  isp: string | null;
  country: string | null;
  city: string | null;
  label: string | null;
  online: boolean;
  checking: boolean;
  paused: boolean;
  last_checked: string | null;
  next_check: string | null;
  since: string | null;
}

export type Theme = "system" | "light" | "dark";

export interface Settings {
  interval_minutes: number;
  notifications: boolean;
  geo_lookup: boolean;
  autostart: boolean;
  start_minimized: boolean;
  track_ipv6: boolean;
  paused: boolean;
  theme: Theme;
  retention_days: number;
  webhook_url: string;
  hook_command: string;
}

export interface HistoryFilter {
  from?: string | null;
  to?: string | null;
  search?: string | null;
  kind?: string | null;
  limit?: number;
  offset?: number;
}

export interface IpStay {
  public_ip: string;
  isp: string | null;
  label: string | null;
  seconds: number;
  count: number;
  first_seen: string;
  last_seen: string;
}

export interface DayCount { day: string; changes: number; offline: number }

export interface Stats {
  days: number;
  total_changes: number;
  offline_events: number;
  unique_ips: number;
  availability_pct: number;
  avg_seconds_between_changes: number | null;
  longest_stay: IpStay | null;
  stays: IpStay[];
  per_day: DayCount[];
  isps: { isp: string; count: number }[];
}

export interface IpLabel { public_ip: string; label: string }

export interface AppInfo {
  version: string;
  data_dir: string;
  db_path: string;
  event_count: number;
  db_size_bytes: number;
}

export const api = {
  current: () => invoke<CurrentStatus>("get_current"),
  checkNow: () => invoke<CurrentStatus>("check_now"),
  setPaused: (paused: boolean) => invoke<boolean>("set_paused", { paused }),
  history: (f: HistoryFilter = {}) => invoke<IpEvent[]>("get_history", { filter: f }),
  historyCount: (f: HistoryFilter = {}) => invoke<number>("get_history_count", { filter: f }),
  deleteEvent: (id: number) => invoke<void>("delete_event", { id }),
  stats: (days: number) => invoke<Stats>("get_stats", { days }),
  labels: () => invoke<IpLabel[]>("get_labels"),
  setLabel: (ip: string, label: string) => invoke<void>("set_label", { ip, label }),
  settings: () => invoke<Settings>("get_settings"),
  saveSettings: (s: Settings) => invoke<Settings>("set_settings", { settings: s }),
  testWebhook: (url: string) => invoke<string>("test_webhook", { url }),
  exportCsv: (path: string, f: HistoryFilter = {}) => invoke<number>("export_csv", { path, filter: f }),
  exportJson: (path: string, f: HistoryFilter = {}) => invoke<number>("export_json", { path, filter: f }),
  clearHistory: () => invoke<void>("clear_history"),
  appInfo: () => invoke<AppInfo>("get_app_info"),
  onEvent: (cb: (e: IpEvent) => void): Promise<UnlistenFn> => listen<IpEvent>("ip-event", (ev) => cb(ev.payload)),
  onChecked: (cb: (s: CurrentStatus) => void): Promise<UnlistenFn> => listen<CurrentStatus>("ip-checked", (ev) => cb(ev.payload)),
  onCopyIp: (cb: (ip: string) => void): Promise<UnlistenFn> => listen<string>("copy-ip", (ev) => cb(ev.payload)),
};
