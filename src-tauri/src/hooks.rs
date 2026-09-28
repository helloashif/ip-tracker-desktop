//! Outbound integrations that fire after an event is recorded.
//! Both are best-effort: failures are logged to stderr and never block tracking.

use crate::db::IpEvent;
use serde::Serialize;

#[derive(Serialize)]
struct WebhookPayload<'a> {
    event: &'a str,
    timestamp: &'a str,
    public_ipv4: Option<&'a str>,
    previous_ipv4: Option<&'a str>,
    public_ipv6: Option<&'a str>,
    previous_ipv6: Option<&'a str>,
    isp: Option<&'a str>,
    country: Option<&'a str>,
    city: Option<&'a str>,
    label: Option<&'a str>,
    local_ips: &'a [String],
}

pub async fn send_webhook(client: &reqwest::Client, url: &str, e: &IpEvent) {
    let payload = WebhookPayload {
        event: &e.kind,
        timestamp: &e.ts,
        public_ipv4: e.public_ip.as_deref(),
        previous_ipv4: e.prev_public_ip.as_deref(),
        public_ipv6: e.public_ipv6.as_deref(),
        previous_ipv6: e.prev_public_ipv6.as_deref(),
        isp: e.isp.as_deref(),
        country: e.country.as_deref(),
        city: e.city.as_deref(),
        label: e.label.as_deref(),
        local_ips: &e.local_ips,
    };
    if let Err(err) = client.post(url).json(&payload).send().await {
        eprintln!("[ip-tracker] webhook failed: {err}");
    }
}

/// Runs a user-supplied shell command with the event exposed as environment variables:
/// IP_EVENT, IP_NEW, IP_OLD, IP_NEW_V6, IP_OLD_V6, IP_ISP, IP_TIMESTAMP.
pub fn run_command(cmd: &str, e: &IpEvent) {
    if cmd.trim().is_empty() {
        return;
    }
    let cmd = cmd.to_string();
    let e = e.clone();
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        let mut c = {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", &cmd]);
            c
        };
        #[cfg(not(target_os = "windows"))]
        let mut c = {
            let mut c = std::process::Command::new("sh");
            c.args(["-c", &cmd]);
            c
        };
        c.env("IP_EVENT", &e.kind)
            .env("IP_TIMESTAMP", &e.ts)
            .env("IP_NEW", e.public_ip.clone().unwrap_or_default())
            .env("IP_OLD", e.prev_public_ip.clone().unwrap_or_default())
            .env("IP_NEW_V6", e.public_ipv6.clone().unwrap_or_default())
            .env("IP_OLD_V6", e.prev_public_ipv6.clone().unwrap_or_default())
            .env("IP_ISP", e.isp.clone().unwrap_or_default())
            .env("IP_LABEL", e.label.clone().unwrap_or_default());
        match c.status() {
            Ok(s) if !s.success() => eprintln!("[ip-tracker] hook command exited with {s}"),
            Err(err) => eprintln!("[ip-tracker] hook command failed to start: {err}"),
            _ => {}
        }
    });
}
