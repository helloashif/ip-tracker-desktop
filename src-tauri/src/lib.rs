mod db;
mod hooks;
mod ip;
mod settings;

use db::{HistoryFilter, IpEvent, IpLabel, NewEvent, Stats};
use serde::Serialize;
use settings::Settings;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Notify;

#[derive(Debug, Clone, Serialize)]
pub struct CurrentStatus {
    pub public_ip: Option<String>,
    pub public_ipv6: Option<String>,
    pub local_ips: Vec<String>,
    pub isp: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub label: Option<String>,
    pub online: bool,
    pub checking: bool,
    pub paused: bool,
    pub last_checked: Option<String>,
    pub next_check: Option<String>,
    pub since: Option<String>,
}

impl Default for CurrentStatus {
    fn default() -> Self {
        Self {
            public_ip: None,
            public_ipv6: None,
            local_ips: Vec::new(),
            isp: None,
            country: None,
            city: None,
            label: None,
            online: false,
            checking: false,
            paused: false,
            last_checked: None,
            next_check: None,
            since: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub db_path: String,
    pub event_count: i64,
    pub db_size_bytes: u64,
}

pub struct AppState {
    data_dir: PathBuf,
    db: Mutex<rusqlite::Connection>,
    settings: Mutex<Settings>,
    current: Mutex<CurrentStatus>,
    client: reqwest::Client,
    wake: Notify,
    check_lock: tokio::sync::Mutex<()>,
}

// ---------- Core check ----------

async fn run_check(app: &AppHandle) -> Result<CurrentStatus, String> {
    let state = app.state::<AppState>();
    let _guard = state.check_lock.lock().await; // never two checks at once

    let (geo_enabled, notify_enabled, track_v6, webhook, hook_cmd, retention) = {
        let s = state.settings.lock().unwrap();
        (s.geo_lookup, s.notifications, s.track_ipv6, s.webhook_url.clone(), s.hook_command.clone(), s.retention_days)
    };

    set_checking(app, true);

    let v4 = ip::public_v4(&state.client).await;
    let v6 = if track_v6 { ip::public_v6(&state.client).await } else { None };
    let locals = ip::local_ips();

    let last = db::latest(&state.db.lock().unwrap())?;

    let mut event: Option<IpEvent> = None;

    match (&v4, &last) {
        (Some(ip), None) => {
            let g = lookup(&state, geo_enabled, ip).await;
            event = Some(record(&state, "start", Some(ip.as_str()), None, v6.as_deref(), None, &locals, &g)?);
        }
        (Some(ip), Some(prev)) if prev.kind == "offline" => {
            let g = lookup(&state, geo_enabled, ip).await;
            let prev_ip = prev.prev_public_ip.clone();
            let prev_v6 = prev.prev_public_ipv6.clone();
            let kind = if prev_ip.as_deref() == Some(ip.as_str()) { "online" } else { "change" };
            event = Some(record(&state, kind, Some(ip.as_str()), prev_ip.as_deref(), v6.as_deref(), prev_v6.as_deref(), &locals, &g)?);
        }
        (Some(ip), Some(prev)) if prev.public_ip.as_deref() != Some(ip.as_str()) || (track_v6 && prev.public_ipv6.is_some() && v6.is_some() && prev.public_ipv6 != v6) => {
            let g = lookup(&state, geo_enabled, ip).await;
            event = Some(record(
                &state, "change", Some(ip.as_str()), prev.public_ip.as_deref(), v6.as_deref(), prev.public_ipv6.as_deref(), &locals, &g,
            )?);
        }
        (None, Some(prev)) if prev.kind != "offline" => {
            event = Some(record(
                &state, "offline", None, prev.public_ip.as_deref(), None, prev.public_ipv6.as_deref(), &locals, &ip::Geo::default(),
            )?);
        }
        _ => {}
    }

    if retention > 0 {
        let _ = db::prune(&state.db.lock().unwrap(), retention);
    }

    let status = {
        let db = state.db.lock().unwrap();
        let latest = db::latest(&db)?;
        let (isp, country, city) = match (&event, &latest) {
            (Some(e), _) => (e.isp.clone(), e.country.clone(), e.city.clone()),
            (None, Some(l)) => (l.isp.clone(), l.country.clone(), l.city.clone()),
            _ => (None, None, None),
        };
        let (since, label) = match &v4 {
            Some(ip) => (db::current_since(&db, ip)?, db::label_for(&db, ip)?),
            None => (None, None),
        };
        let paused = state.settings.lock().unwrap().paused;
        CurrentStatus {
            public_ip: v4.clone(),
            public_ipv6: v6.clone(),
            local_ips: locals.clone(),
            isp,
            country,
            city,
            label,
            online: v4.is_some(),
            checking: false,
            paused,
            last_checked: Some(db::now_str()),
            next_check: next_check_time(&state),
            since,
        }
    };

    *state.current.lock().unwrap() = status.clone();
    update_tray(app, &status);
    let _ = app.emit("ip-checked", &status);

    if let Some(e) = event {
        let _ = app.emit("ip-event", &e);
        if e.kind != "start" {
            if notify_enabled {
                notify(app, &e);
            }
            if !webhook.trim().is_empty() {
                let client = state.client.clone();
                let ev = e.clone();
                tauri::async_runtime::spawn(async move { hooks::send_webhook(&client, &webhook, &ev).await });
            }
            hooks::run_command(&hook_cmd, &e);
        }
    }

    Ok(status)
}

async fn lookup(state: &State<'_, AppState>, enabled: bool, ip: &str) -> ip::Geo {
    if enabled { ip::geo(&state.client, ip).await } else { ip::Geo::default() }
}

fn next_check_time(state: &State<AppState>) -> Option<String> {
    let s = state.settings.lock().unwrap();
    if s.paused {
        return None;
    }
    let t = chrono::Utc::now() + chrono::Duration::seconds(s.interval_secs() as i64);
    Some(t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn set_checking(app: &AppHandle, checking: bool) {
    let state = app.state::<AppState>();
    let mut cur = state.current.lock().unwrap();
    cur.checking = checking;
    let _ = app.emit("ip-checked", &*cur);
}

#[allow(clippy::too_many_arguments)]
fn record(
    state: &State<AppState>,
    kind: &str,
    public_ip: Option<&str>,
    prev_public_ip: Option<&str>,
    public_ipv6: Option<&str>,
    prev_public_ipv6: Option<&str>,
    locals: &[String],
    g: &ip::Geo,
) -> Result<IpEvent, String> {
    let db = state.db.lock().unwrap();
    db::insert(
        &db,
        &NewEvent {
            kind,
            public_ip,
            prev_public_ip,
            public_ipv6,
            prev_public_ipv6,
            local_ips: locals,
            isp: g.isp.as_deref(),
            country: g.country.as_deref(),
            city: g.city.as_deref(),
        },
    )
}

fn notify(app: &AppHandle, e: &IpEvent) {
    let (title, body) = match e.kind.as_str() {
        "change" => {
            let old = e.prev_public_ip.clone().unwrap_or_else(|| "?".into());
            let new = e.public_ip.clone().unwrap_or_default();
            let mut body = format!("{old} → {new}");
            if let Some(isp) = &e.isp {
                body.push_str(&format!("\n{isp}"));
            }
            ("Public IP changed".to_string(), body)
        }
        "offline" => ("Offline".to_string(), "Could not reach any IP lookup service.".to_string()),
        "online" => ("Back online".to_string(), e.public_ip.clone().unwrap_or_default()),
        _ => (e.kind.clone(), String::new()),
    };
    let _ = app.notification().builder().title(title).body(body).show();
}

async fn poll_loop(app: AppHandle) {
    loop {
        let paused = app.state::<AppState>().settings.lock().unwrap().paused;
        if !paused {
            let _ = run_check(&app).await;
        }
        let secs = app.state::<AppState>().settings.lock().unwrap().interval_secs();
        let state = app.state::<AppState>();
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => {}
            _ = state.wake.notified() => {}
        }
    }
}

// ---------- Tray ----------

struct TrayHandle(TrayIcon);
struct TrayIpItem(MenuItem<tauri::Wry>);
struct TrayPauseItem(MenuItem<tauri::Wry>);

fn update_tray(app: &AppHandle, s: &CurrentStatus) {
    if let Some(tray) = app.try_state::<TrayHandle>() {
        let text = match (&s.public_ip, s.paused) {
            (_, true) => "IP Tracker — paused".to_string(),
            (Some(ip), _) => format!("IP Tracker — {ip}"),
            (None, _) => "IP Tracker — offline".to_string(),
        };
        let _ = tray.0.set_tooltip(Some(&text));
        #[cfg(target_os = "macos")]
        {
            let _ = tray.0.set_title(s.public_ip.as_deref());
        }
    }
    if let Some(item) = app.try_state::<TrayIpItem>() {
        let _ = item.0.set_text(match &s.public_ip {
            Some(ip) => format!("Copy {ip}"),
            None => "Offline".to_string(),
        });
    }
    if let Some(item) = app.try_state::<TrayPauseItem>() {
        let _ = item.0.set_text(if s.paused { "Resume tracking" } else { "Pause tracking" });
    }
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let ip_item = MenuItem::with_id(app, "copy_ip", "Checking…", true, None::<&str>)?;
    let check = MenuItem::with_id(app, "check", "Check now", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause tracking", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open IP Tracker", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&ip_item, &sep, &check, &pause, &open, &sep2, &quit])?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("default icon"))
        .tooltip("IP Tracker")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "copy_ip" => {
                let ip = app.state::<AppState>().current.lock().unwrap().public_ip.clone();
                if let Some(ip) = ip {
                    let _ = app.emit("copy-ip", ip);
                }
            }
            "check" => app.state::<AppState>().wake.notify_one(),
            "pause" => {
                let _ = toggle_pause(app);
            }
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayHandle(tray));
    app.manage(TrayIpItem(ip_item));
    app.manage(TrayPauseItem(pause));
    Ok(())
}

fn toggle_pause(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let paused = {
        let mut s = state.settings.lock().unwrap();
        s.paused = !s.paused;
        s.save(&state.data_dir)?;
        s.paused
    };
    let status = {
        let mut cur = state.current.lock().unwrap();
        cur.paused = paused;
        cur.next_check = if paused { None } else { next_check_time(&state) };
        cur.clone()
    };
    update_tray(app, &status);
    let _ = app.emit("ip-checked", &status);
    state.wake.notify_one();
    Ok(paused)
}

// ---------- Commands ----------

#[tauri::command]
fn get_current(state: State<AppState>) -> CurrentStatus {
    state.current.lock().unwrap().clone()
}

#[tauri::command]
async fn check_now(app: AppHandle) -> Result<CurrentStatus, String> {
    let s = run_check(&app).await?;
    app.state::<AppState>().wake.notify_one(); // restart the interval from now
    Ok(s)
}

#[tauri::command]
fn set_paused(app: AppHandle, paused: bool) -> Result<bool, String> {
    let current = app.state::<AppState>().settings.lock().unwrap().paused;
    if current == paused {
        return Ok(paused);
    }
    toggle_pause(&app)
}

#[tauri::command]
fn get_history(state: State<AppState>, filter: HistoryFilter) -> Result<Vec<IpEvent>, String> {
    db::history(&state.db.lock().unwrap(), &filter)
}

#[tauri::command]
fn get_history_count(state: State<AppState>, filter: HistoryFilter) -> Result<i64, String> {
    db::history_count(&state.db.lock().unwrap(), &filter)
}

#[tauri::command]
fn delete_event(state: State<AppState>, id: i64) -> Result<(), String> {
    db::delete_event(&state.db.lock().unwrap(), id)
}

#[tauri::command]
fn get_stats(state: State<AppState>, days: i64) -> Result<Stats, String> {
    db::stats(&state.db.lock().unwrap(), days)
}

#[tauri::command]
fn get_labels(state: State<AppState>) -> Result<Vec<IpLabel>, String> {
    db::labels(&state.db.lock().unwrap())
}

#[tauri::command]
fn set_label(app: AppHandle, state: State<AppState>, ip: String, label: String) -> Result<(), String> {
    db::set_label(&state.db.lock().unwrap(), &ip, &label)?;
    let mut cur = state.current.lock().unwrap();
    if cur.public_ip.as_deref() == Some(ip.as_str()) {
        cur.label = if label.trim().is_empty() { None } else { Some(label.trim().to_string()) };
        let _ = app.emit("ip-checked", &*cur);
    }
    Ok(())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn set_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<Settings, String> {
    settings.save(&state.data_dir)?;
    *state.settings.lock().unwrap() = settings.clone();
    let status = {
        let mut cur = state.current.lock().unwrap();
        cur.paused = settings.paused;
        cur.next_check = next_check_time(&state);
        cur.clone()
    };
    update_tray(&app, &status);
    let _ = app.emit("ip-checked", &status);
    state.wake.notify_one();
    Ok(settings)
}

#[tauri::command]
async fn test_webhook(state: State<'_, AppState>, url: String) -> Result<String, String> {
    let e = IpEvent {
        id: 0,
        ts: db::now_str(),
        kind: "test".into(),
        public_ip: Some("203.0.113.42".into()),
        prev_public_ip: Some("203.0.113.41".into()),
        public_ipv6: None,
        prev_public_ipv6: None,
        local_ips: vec!["192.168.1.10".into()],
        isp: Some("Example ISP".into()),
        country: Some("Example".into()),
        city: None,
        label: None,
    };
    let resp = state.client.post(&url).json(&serde_json::json!({
        "event": e.kind, "timestamp": e.ts, "public_ipv4": e.public_ip, "previous_ipv4": e.prev_public_ip,
        "isp": e.isp, "country": e.country, "local_ips": e.local_ips, "test": true
    })).send().await.map_err(|e| e.to_string())?;
    Ok(format!("HTTP {}", resp.status().as_u16()))
}

#[tauri::command]
fn export_csv(state: State<AppState>, path: String, filter: HistoryFilter) -> Result<usize, String> {
    db::export_csv(&state.db.lock().unwrap(), std::path::Path::new(&path), &filter)
}

#[tauri::command]
fn export_json(state: State<AppState>, path: String, filter: HistoryFilter) -> Result<usize, String> {
    db::export_json(&state.db.lock().unwrap(), std::path::Path::new(&path), &filter)
}

#[tauri::command]
fn clear_history(state: State<AppState>) -> Result<(), String> {
    db::clear(&state.db.lock().unwrap())
}

#[tauri::command]
fn get_app_info(app: AppHandle, state: State<AppState>) -> Result<AppInfo, String> {
    let db_path = state.data_dir.join("ip-tracker.sqlite");
    let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let count = db::history_count(&state.db.lock().unwrap(), &HistoryFilter::default())?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        data_dir: state.data_dir.display().to_string(),
        db_path: db_path.display().to_string(),
        event_count: count,
        db_size_bytes: size,
    })
}

// ---------- Entry ----------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let settings = Settings::load(&data_dir);
            let conn = db::open(&data_dir).map_err(std::io::Error::other)?;

            let start_minimized = settings.start_minimized || std::env::args().any(|a| a == "--minimized");
            let paused = settings.paused;

            app.manage(AppState {
                data_dir,
                db: Mutex::new(conn),
                settings: Mutex::new(settings),
                current: Mutex::new(CurrentStatus { paused, ..Default::default() }),
                client: ip::client(),
                wake: Notify::new(),
                check_lock: tokio::sync::Mutex::new(()),
            });

            build_tray(app.handle())?;

            if !start_minimized {
                show_main(app.handle());
            }

            if let Some(w) = app.get_webview_window("main") {
                let w2 = w.clone();
                w.on_window_event(move |e| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                        api.prevent_close();
                        let _ = w2.hide();
                    }
                });
            }

            tauri::async_runtime::spawn(poll_loop(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_current,
            check_now,
            set_paused,
            get_history,
            get_history_count,
            delete_event,
            get_stats,
            get_labels,
            set_label,
            get_settings,
            set_settings,
            test_webhook,
            export_csv,
            export_json,
            clear_history,
            get_app_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running IP Tracker");
}
