use std::collections::HashMap;
use std::time::Instant;

use reqwest::{redirect, Version};
use tracing::debug;

use crate::{
    error::{AnalyzerError, Result},
    types::{CookieInfo, HttpVersion, SiteData},
};

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; SiteAnalyzer/0.1; +https://github.com/site-analyzer)";
const DEFAULT_TIMEOUT_SECS: u64 = 15;

pub struct Fetcher {
    client: reqwest::Client,
}

impl Fetcher {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .redirect(redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .cookie_store(true)
            .build()
            .map_err(AnalyzerError::HttpError)?;

        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<SiteData> {
        let start = Instant::now();
        debug!("Fetching {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Cache-Control", "no-cache")
            .send()
            .await
            .map_err(AnalyzerError::HttpError)?;

        let response_time_ms = start.elapsed().as_millis() as u64;
        let final_url = response.url().to_string();
        let status_code = response.status().as_u16();

        // HTTP version
        let http_version = match response.version() {
            Version::HTTP_10 => HttpVersion::Http1,
            Version::HTTP_11 => HttpVersion::Http11,
            Version::HTTP_2  => HttpVersion::Http2,
            Version::HTTP_3  => HttpVersion::Http3,
            _                => HttpVersion::Http11,
        };

        // Collect headers (lowercase keys)
        let mut headers: HashMap<String, String> = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.as_str().to_lowercase(), v.to_string());
            }
        }

        // Upgrade to HTTP/3 if Alt-Svc advertises h3
        let http_version = if http_version != HttpVersion::Http3 {
            if headers.get("alt-svc").map(|v| v.contains("h3")).unwrap_or(false) {
                HttpVersion::Http3
            } else {
                http_version
            }
        } else {
            http_version
        };

        // Cookies from reqwest's cookie jar
        let cookies: Vec<CookieInfo> = response
            .cookies()
            .map(|c| {
                let max_age_seconds = c.max_age().map(|d| d.as_secs() as i64);
                // If no max-age but has expires, treat as session-adjacent
                CookieInfo {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    http_only: c.http_only(),
                    secure: c.secure(),
                    max_age_seconds,
                    same_site: None, // reqwest doesn't expose SameSite
                    domain: c.domain().map(|s| s.to_string()),
                }
            })
            .collect();

        let html = response.text().await.unwrap_or_default();

        Ok(SiteData {
            url: url.to_string(),
            final_url,
            html,
            headers,
            cookies,
            status_code,
            http_version,
            response_time_ms,
            ip_address: None,
            dns_records: Default::default(),
            tls_info: None,
        })
    }
}
