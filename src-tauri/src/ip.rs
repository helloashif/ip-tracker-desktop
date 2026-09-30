use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

const V4_ENDPOINTS: &[&str] =
    &["https://api4.ipify.org", "https://ipv4.icanhazip.com", "https://ifconfig.me/ip", "https://ipinfo.io/ip"];

const V6_ENDPOINTS: &[&str] = &["https://api6.ipify.org", "https://ipv6.icanhazip.com"];

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(concat!("ip-tracker/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("reqwest client")
}

async fn fetch_ip<T: std::str::FromStr>(client: &reqwest::Client, endpoints: &[&str]) -> Option<String> {
    for url in endpoints {
        if let Ok(resp) = client.get(*url).send().await {
            if let Ok(text) = resp.text().await {
                let t = text.trim();
                if t.parse::<T>().is_ok() {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

/// Public IPv4. `None` means no endpoint answered — treated as offline.
pub async fn public_v4(client: &reqwest::Client) -> Option<String> {
    fetch_ip::<Ipv4Addr>(client, V4_ENDPOINTS).await
}

/// Public IPv6. `None` just means no IPv6 connectivity; not an offline signal.
pub async fn public_v6(client: &reqwest::Client) -> Option<String> {
    fetch_ip::<Ipv6Addr>(client, V6_ENDPOINTS).await
}

#[derive(Debug, Clone, Default)]
pub struct Geo {
    pub isp: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

#[derive(Deserialize, Default)]
struct IpWhoIsConnection {
    #[serde(default)]
    isp: Option<String>,
    #[serde(default)]
    org: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoIsResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    connection: Option<IpWhoIsConnection>,
}

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    #[serde(default)]
    isp: Option<String>,
    #[serde(default)]
    org: Option<String>,
    #[serde(default)]
    #[serde(rename = "as")]
    as_field: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    city: Option<String>,
}

fn clean_as_name(as_str: &str) -> Option<String> {
    let trimmed = as_str.trim();
    if let Some(rest) = trimmed.strip_prefix("AS") {
        let after_num = rest.trim_start_matches(|c: char| c.is_ascii_digit()).trim();
        if !after_num.is_empty() {
            return Some(after_num.to_string());
        }
    }
    if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Best-effort ISP / location lookup.
/// Primary: ipwho.is (HTTPS, accurate ASN & ISP mapping, free).
/// Fallback: ip-api.com (includes AS organization fallback when APNIC descriptor lists an individual).
pub async fn geo(client: &reqwest::Client, ip: &str) -> Geo {
    // 1. Try ipwho.is (accurate ISP and organization mapping)
    let whois_url = format!("https://ipwho.is/{ip}");
    if let Ok(resp) = client.get(&whois_url).send().await {
        if let Ok(r) = resp.json::<IpWhoIsResponse>().await {
            if r.success {
                let conn = r.connection.unwrap_or_default();
                let isp = conn.isp.filter(|s| !s.is_empty()).or(conn.org.filter(|s| !s.is_empty()));
                return Geo {
                    isp,
                    country: r.country.filter(|s| !s.is_empty()),
                    city: r.city.filter(|s| !s.is_empty()),
                };
            }
        }
    }

    // 2. Fallback to ip-api.com (with AS organization cleanup)
    let api_url = format!("http://ip-api.com/json/{ip}?fields=status,isp,org,as,country,city");
    if let Ok(resp) = client.get(&api_url).send().await {
        if let Ok(r) = resp.json::<IpApiResponse>().await {
            if r.status == "success" {
                let as_org = r.as_field.as_deref().and_then(clean_as_name);
                let isp =
                    as_org.or_else(|| r.org.filter(|s| !s.is_empty())).or_else(|| r.isp.filter(|s| !s.is_empty()));

                return Geo {
                    isp,
                    country: r.country.filter(|s| !s.is_empty()),
                    city: r.city.filter(|s| !s.is_empty()),
                };
            }
        }
    }

    Geo::default()
}

/// Non-loopback local addresses, IPv4 first, then global IPv6.
pub fn local_ips() -> Vec<String> {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    if let Ok(list) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in list {
            match ip {
                IpAddr::V4(a) if !a.is_loopback() && !a.is_link_local() => v4.push(a.to_string()),
                IpAddr::V6(a) if !a.is_loopback() && (a.segments()[0] & 0xffc0) != 0xfe80 => v6.push(a.to_string()),
                _ => {}
            }
        }
    }
    v4.sort();
    v4.dedup();
    v6.sort();
    v6.dedup();
    v4.extend(v6);
    v4
}
