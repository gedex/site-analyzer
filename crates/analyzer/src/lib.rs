pub mod category_map;
pub mod dns;
pub mod engine;
pub mod error;
pub mod fetcher;
pub mod fingerprints;
pub mod pattern;
pub mod tls;
pub mod types;

// Supplemental detectors (site-element analysis not in Wappalyzer data)
pub mod detectors;

use tracing::{debug, info, warn};
use url::Url;

use crate::{
    detectors::html::HtmlDetector,
    detectors::Detector,
    dns::DnsResolver,
    engine::run_engine,
    error::{AnalyzerError, Result},
    fetcher::Fetcher,
    tls::fetch_tls_info,
    types::AnalysisResult,
};

// Re-export public types
pub use fingerprints::FingerprintDb;
pub use types::{AnalysisResult as SiteAnalysis, TechCategory, Technology};

/// Configuration for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Whether to perform DNS resolution
    pub resolve_dns: bool,
    /// Whether to fetch TLS certificate info
    pub fetch_tls: bool,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Minimum confidence threshold (0-100); detections below this are dropped
    pub min_confidence: u8,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            resolve_dns: true,
            fetch_tls: true,
            timeout_secs: 15,
            min_confidence: 50,
        }
    }
}

/// Main analyzer
pub struct Analyzer {
    config: AnalyzerConfig,
    fetcher: Fetcher,
    supplemental: Vec<Box<dyn Detector>>,
}

impl Analyzer {
    pub fn new() -> Result<Self> {
        Self::with_config(AnalyzerConfig::default())
    }

    pub fn with_config(config: AnalyzerConfig) -> Result<Self> {
        let fetcher = Fetcher::new()?;
        let supplemental: Vec<Box<dyn Detector>> = vec![Box::new(HtmlDetector)];
        Ok(Self { config, fetcher, supplemental })
    }

    pub async fn analyze(&self, url: &str) -> Result<SiteAnalysis> {
        let url = normalize_url(url)?;
        let parsed = Url::parse(&url)?;
        let host = parsed
            .host_str()
            .ok_or_else(|| AnalyzerError::InvalidDomain(url.clone()))?
            .to_string();
        let domain = root_domain(&host);

        info!("Analyzing {}", url);

        // 1. Fetch
        let mut site_data = self.fetcher.fetch(&url).await?;

        // 2. DNS
        if self.config.resolve_dns {
            debug!("Resolving DNS for {}", domain);
            let resolver = DnsResolver::new().await?;
            let dns = resolver.resolve(&host).await;
            site_data.ip_address = dns.a_records.first().cloned();
            site_data.dns_records = dns;
        }

        // 3. TLS
        if self.config.fetch_tls && url.starts_with("https://") {
            let port = parsed.port().unwrap_or(443);
            debug!("Fetching TLS info for {}:{}", host, port);
            site_data.tls_info = fetch_tls_info(&host, port).await;
        }

        // 4. Wappalyzer engine
        let mut result = AnalysisResult::new(&domain);
        if let Err(e) = run_engine(&site_data, &mut result).await {
            warn!("Fingerprint engine error: {}", e);
        }

        // 5. Supplemental detectors
        for detector in &self.supplemental {
            debug!("Running supplemental detector: {}", detector.name());
            if let Err(e) = detector.detect(&site_data, &mut result).await {
                warn!("Detector {} failed: {}", detector.name(), e);
            }
        }

        // 6. Post-processing
        result.dedup();
        apply_confidence_filter(&mut result, self.config.min_confidence);

        Ok(result)
    }
}

fn apply_confidence_filter(result: &mut AnalysisResult, min_confidence: u8) {
    if min_confidence == 0 { return; }
    let threshold = min_confidence as f32 / 100.0;
    for techs in result.categories.values_mut() {
        techs.retain(|t| t.confidence.map(|c| c >= threshold).unwrap_or(true));
    }
    result.categories.retain(|_, v| !v.is_empty());
}

fn normalize_url(input: &str) -> Result<String> {
    let s = input.trim();
    if s.starts_with("http://") || s.starts_with("https://") {
        Ok(s.to_string())
    } else {
        Ok(format!("https://{}", s))
    }
}

fn root_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        parts[parts.len() - 2..].join(".")
    } else {
        host.to_string()
    }
}
