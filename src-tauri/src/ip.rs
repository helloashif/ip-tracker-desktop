use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

const V4_ENDPOINTS: &[&str] = &[
    "https://api4.ipify.org",
    "https://ipv4.icanhazip.com",
    "https://ifconfig.me/ip",
    "https://ipinfo.io/ip",
];

const V6_ENDPOINTS: &[&str] = &[
    "https://api6.ipify.org",
    "https://ipv6.icanhazip.com",
];

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

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    #[serde(default)]
    isp: Option<String>,
    #[serde(default)]
    org: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    city: Option<String>,
}

/// Best-effort ISP / location lookup via ip-api.com (free, 45 req/min).
pub async fn geo(client: &reqwest::Client, ip: &str) -> Geo {
    let url = format!("http://ip-api.com/json/{ip}?fields=status,isp,org,country,city");
    match client.get(url).send().await {
        Ok(resp) => match resp.json::<IpApiResponse>().await {
            Ok(r) if r.status == "success" => Geo {
                isp: r.isp.filter(|s| !s.is_empty()).or(r.org),
                country: r.country,
                city: r.city,
            },
            _ => Geo::default(),
        },
        Err(_) => Geo::default(),
    }
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
