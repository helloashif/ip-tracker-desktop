use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpEvent {
    pub id: i64,
    pub ts: String,
    pub kind: String,
    pub public_ip: Option<String>,
    pub prev_public_ip: Option<String>,
    pub public_ipv6: Option<String>,
    pub prev_public_ipv6: Option<String>,
    pub local_ips: Vec<String>,
    pub isp: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct HistoryFilter {
    pub from: Option<String>,
    pub to: Option<String>,
    pub search: Option<String>,
    pub kind: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IpStay {
    pub public_ip: String,
    pub isp: Option<String>,
    pub label: Option<String>,
    pub seconds: i64,
    pub count: i64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayCount {
    pub day: String,
    pub changes: i64,
    pub offline: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IspCount {
    pub isp: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub days: i64,
    pub total_changes: i64,
    pub offline_events: i64,
    pub unique_ips: i64,
    pub availability_pct: f64,
    pub avg_seconds_between_changes: Option<f64>,
    pub longest_stay: Option<IpStay>,
    pub stays: Vec<IpStay>,
    pub per_day: Vec<DayCount>,
    pub isps: Vec<IspCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpLabel {
    pub public_ip: String,
    pub label: String,
}

pub struct NewEvent<'a> {
    pub kind: &'a str,
    pub public_ip: Option<&'a str>,
    pub prev_public_ip: Option<&'a str>,
    pub public_ipv6: Option<&'a str>,
    pub prev_public_ipv6: Option<&'a str>,
    pub local_ips: &'a [String],
    pub isp: Option<&'a str>,
    pub country: Option<&'a str>,
    pub city: Option<&'a str>,
}

pub fn open(dir: &Path) -> Result<Connection, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let conn = Connection::open(dir.join("ip-tracker.sqlite")).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS ip_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts TEXT NOT NULL,
            kind TEXT NOT NULL,
            public_ip TEXT,
            prev_public_ip TEXT,
            local_ips TEXT NOT NULL DEFAULT '[]',
            isp TEXT,
            country TEXT,
            city TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_ip_events_ts ON ip_events(ts);
         CREATE INDEX IF NOT EXISTS idx_ip_events_public_ip ON ip_events(public_ip);
         CREATE TABLE IF NOT EXISTS ip_labels (
            public_ip TEXT PRIMARY KEY,
            label TEXT NOT NULL
         );",
    )
    .map_err(|e| e.to_string())?;
    migrate(&conn)?;
    Ok(conn)
}

/// Additive migrations: add columns that older databases lack.
fn migrate(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn.prepare("PRAGMA table_info(ip_events)").map_err(|e| e.to_string())?;
    let cols: Vec<String> =
        stmt.query_map([], |r| r.get::<_, String>(1)).map_err(|e| e.to_string())?.filter_map(Result::ok).collect();
    for col in ["public_ipv6", "prev_public_ipv6"] {
        if !cols.iter().any(|c| c == col) {
            conn.execute(&format!("ALTER TABLE ip_events ADD COLUMN {col} TEXT"), []).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

const COLS: &str = "e.id, e.ts, e.kind, e.public_ip, e.prev_public_ip, e.local_ips, e.isp, e.country, e.city, \
                    e.public_ipv6, e.prev_public_ipv6, l.label";
const FROM: &str = "FROM ip_events e LEFT JOIN ip_labels l ON l.public_ip = e.public_ip";

pub fn insert(conn: &Connection, e: &NewEvent) -> Result<IpEvent, String> {
    let ts = now_str();
    let local = serde_json::to_string(e.local_ips).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO ip_events (ts, kind, public_ip, prev_public_ip, local_ips, isp, country, city, public_ipv6, prev_public_ipv6)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![ts, e.kind, e.public_ip, e.prev_public_ip, local, e.isp, e.country, e.city, e.public_ipv6, e.prev_public_ipv6],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or_else(|| "insert vanished".into())
}

fn get(conn: &Connection, id: i64) -> Result<Option<IpEvent>, String> {
    conn.query_row(&format!("SELECT {COLS} {FROM} WHERE e.id = ?1"), params![id], row_to_event)
        .optional()
        .map_err(|e| e.to_string())
}

pub fn now_str() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<IpEvent> {
    let local: String = row.get(5)?;
    Ok(IpEvent {
        id: row.get(0)?,
        ts: row.get(1)?,
        kind: row.get(2)?,
        public_ip: row.get(3)?,
        prev_public_ip: row.get(4)?,
        local_ips: serde_json::from_str(&local).unwrap_or_default(),
        isp: row.get(6)?,
        country: row.get(7)?,
        city: row.get(8)?,
        public_ipv6: row.get(9)?,
        prev_public_ipv6: row.get(10)?,
        label: row.get(11)?,
    })
}

pub fn latest(conn: &Connection) -> Result<Option<IpEvent>, String> {
    conn.query_row(&format!("SELECT {COLS} {FROM} ORDER BY e.id DESC LIMIT 1"), [], row_to_event)
        .optional()
        .map_err(|e| e.to_string())
}

pub fn current_since(conn: &Connection, ip: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT ts FROM ip_events WHERE public_ip = ?1 AND kind IN ('start','change','online') ORDER BY id DESC LIMIT 1",
        params![ip],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn where_clause(f: &HistoryFilter) -> (String, Vec<rusqlite::types::Value>) {
    let mut clauses = Vec::new();
    let mut args: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(from) = &f.from {
        clauses.push("e.ts >= ?".to_string());
        args.push(from.clone().into());
    }
    if let Some(to) = &f.to {
        clauses.push("e.ts < ?".to_string());
        args.push(to.clone().into());
    }
    if let Some(k) = &f.kind {
        if !k.is_empty() && k != "all" {
            clauses.push("e.kind = ?".to_string());
            args.push(k.clone().into());
        }
    }
    if let Some(s) = &f.search {
        if !s.trim().is_empty() {
            let like = format!("%{}%", s.trim());
            clauses.push(
                "(e.public_ip LIKE ? OR e.prev_public_ip LIKE ? OR e.public_ipv6 LIKE ? OR e.isp LIKE ? OR e.city LIKE ? OR e.country LIKE ? OR l.label LIKE ?)".to_string(),
            );
            for _ in 0..7 {
                args.push(like.clone().into());
            }
        }
    }
    let w = if clauses.is_empty() { String::new() } else { format!("WHERE {}", clauses.join(" AND ")) };
    (w, args)
}

pub fn history(conn: &Connection, f: &HistoryFilter) -> Result<Vec<IpEvent>, String> {
    let (w, args) = where_clause(f);
    let limit = f.limit.unwrap_or(100).clamp(1, 10_000);
    let offset = f.offset.unwrap_or(0).max(0);
    let sql = format!("SELECT {COLS} {FROM} {w} ORDER BY e.id DESC LIMIT {limit} OFFSET {offset}");
    query_events(conn, &sql, args)
}

pub fn history_all(conn: &Connection, f: &HistoryFilter) -> Result<Vec<IpEvent>, String> {
    let (w, args) = where_clause(f);
    let sql = format!("SELECT {COLS} {FROM} {w} ORDER BY e.id ASC");
    query_events(conn, &sql, args)
}

fn query_events(conn: &Connection, sql: &str, args: Vec<rusqlite::types::Value>) -> Result<Vec<IpEvent>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params_from_iter(args), row_to_event).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn history_count(conn: &Connection, f: &HistoryFilter) -> Result<i64, String> {
    let (w, args) = where_clause(f);
    conn.query_row(&format!("SELECT COUNT(*) {FROM} {w}"), rusqlite::params_from_iter(args), |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn clear(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM ip_events", []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_event(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM ip_events WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove events older than `days`, but never the most recent one (it holds current state).
pub fn prune(conn: &Connection, days: u32) -> Result<usize, String> {
    if days == 0 {
        return Ok(0);
    }
    let cutoff = (Utc::now() - Duration::days(days as i64)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    conn.execute("DELETE FROM ip_events WHERE ts < ?1 AND id < (SELECT MAX(id) FROM ip_events)", params![cutoff])
        .map_err(|e| e.to_string())
}

pub fn labels(conn: &Connection) -> Result<Vec<IpLabel>, String> {
    let mut stmt = conn.prepare("SELECT public_ip, label FROM ip_labels ORDER BY label").map_err(|e| e.to_string())?;
    let rows =
        stmt.query_map([], |r| Ok(IpLabel { public_ip: r.get(0)?, label: r.get(1)? })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn set_label(conn: &Connection, ip: &str, label: &str) -> Result<(), String> {
    let label = label.trim();
    if label.is_empty() {
        conn.execute("DELETE FROM ip_labels WHERE public_ip = ?1", params![ip]).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO ip_labels (public_ip, label) VALUES (?1, ?2) ON CONFLICT(public_ip) DO UPDATE SET label = excluded.label",
            params![ip, label],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn label_for(conn: &Connection, ip: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT label FROM ip_labels WHERE public_ip = ?1", params![ip], |r| r.get(0))
        .optional()
        .map_err(|e| e.to_string())
}

pub fn stats(conn: &Connection, days: i64) -> Result<Stats, String> {
    let days = days.clamp(1, 3650);
    let now = Utc::now();
    let from = now - Duration::days(days);
    let from_str = from.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let mut events = query_events(
        conn,
        &format!("SELECT {COLS} {FROM} WHERE e.ts >= ?1 ORDER BY e.id ASC"),
        vec![from_str.clone().into()],
    )?;
    let before = conn
        .query_row(
            &format!("SELECT {COLS} {FROM} WHERE e.ts < ?1 ORDER BY e.id DESC LIMIT 1"),
            params![from_str],
            row_to_event,
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let total_changes = events.iter().filter(|e| e.kind == "change").count() as i64;
    let offline_events = events.iter().filter(|e| e.kind == "offline").count() as i64;

    let mut per_day_map: HashMap<String, (i64, i64)> = HashMap::new();
    for e in &events {
        let entry = per_day_map.entry(e.ts[..10].to_string()).or_insert((0, 0));
        match e.kind.as_str() {
            "change" => entry.0 += 1,
            "offline" => entry.1 += 1,
            _ => {}
        }
    }
    let mut per_day = Vec::new();
    for i in (0..days).rev() {
        let day = (now - Duration::days(i)).format("%Y-%m-%d").to_string();
        let (c, o) = per_day_map.get(&day).copied().unwrap_or((0, 0));
        per_day.push(DayCount { day, changes: c, offline: o });
    }

    if let Some(mut b) = before {
        b.ts = from_str.clone();
        events.insert(0, b);
    }

    let mut stays: HashMap<String, IpStay> = HashMap::new();
    let mut isps: HashMap<String, i64> = HashMap::new();
    let mut gaps: Vec<f64> = Vec::new();
    let mut last_change_ts: Option<DateTime<Utc>> = None;
    let mut offline_secs: i64 = 0;
    let mut tracked_secs: i64 = 0;

    for i in 0..events.len() {
        let e = &events[i];
        let start = parse_ts(&e.ts).unwrap_or(now);
        let end = events.get(i + 1).and_then(|n| parse_ts(&n.ts)).unwrap_or(now);
        let secs = (end - start).num_seconds().max(0);
        tracked_secs += secs;

        if let Some(isp) = &e.isp {
            *isps.entry(isp.clone()).or_insert(0) += 1;
        }
        if e.kind == "change" {
            if let Some(prev) = last_change_ts {
                gaps.push((start - prev).num_seconds() as f64);
            }
            last_change_ts = Some(start);
        }
        if e.kind == "offline" {
            offline_secs += secs;
            continue;
        }
        if let Some(ip) = &e.public_ip {
            let s = stays.entry(ip.clone()).or_insert_with(|| IpStay {
                public_ip: ip.clone(),
                isp: e.isp.clone(),
                label: e.label.clone(),
                seconds: 0,
                count: 0,
                first_seen: e.ts.clone(),
                last_seen: e.ts.clone(),
            });
            s.seconds += secs;
            s.last_seen = e.ts.clone();
            if s.isp.is_none() {
                s.isp = e.isp.clone();
            }
            if matches!(e.kind.as_str(), "change" | "start" | "online") {
                s.count += 1;
            }
        }
    }

    let mut stays: Vec<IpStay> = stays.into_values().collect();
    stays.sort_by_key(|a| std::cmp::Reverse(a.seconds));
    let longest_stay = stays.first().cloned();
    let unique_ips = stays.len() as i64;
    let avg = if gaps.is_empty() { None } else { Some(gaps.iter().sum::<f64>() / gaps.len() as f64) };
    let availability_pct =
        if tracked_secs > 0 { ((tracked_secs - offline_secs) as f64 / tracked_secs as f64) * 100.0 } else { 100.0 };

    let mut isps: Vec<IspCount> = isps.into_iter().map(|(isp, count)| IspCount { isp, count }).collect();
    isps.sort_by_key(|a| std::cmp::Reverse(a.count));

    Ok(Stats {
        days,
        total_changes,
        offline_events,
        unique_ips,
        availability_pct,
        avg_seconds_between_changes: avg,
        longest_stay,
        stays,
        per_day,
        isps,
    })
}

fn parse_ts(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}

pub fn export_csv(conn: &Connection, path: &Path, f: &HistoryFilter) -> Result<usize, String> {
    let rows = history_all(conn, f)?;
    let mut out = String::from(
        "timestamp,event,public_ipv4,previous_ipv4,public_ipv6,previous_ipv6,label,isp,country,city,local_ips\n",
    );
    for e in &rows {
        let cells = [
            e.ts.clone(),
            e.kind.clone(),
            e.public_ip.clone().unwrap_or_default(),
            e.prev_public_ip.clone().unwrap_or_default(),
            e.public_ipv6.clone().unwrap_or_default(),
            e.prev_public_ipv6.clone().unwrap_or_default(),
            e.label.clone().unwrap_or_default(),
            e.isp.clone().unwrap_or_default(),
            e.country.clone().unwrap_or_default(),
            e.city.clone().unwrap_or_default(),
            e.local_ips.join(" "),
        ];
        let line: Vec<String> = cells.iter().map(|c| csv_cell(c)).collect();
        out.push_str(&line.join(","));
        out.push('\n');
    }
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(rows.len())
}

pub fn export_json(conn: &Connection, path: &Path, f: &HistoryFilter) -> Result<usize, String> {
    let rows = history_all(conn, f)?;
    let json = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(rows.len())
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
