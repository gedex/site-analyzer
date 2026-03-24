use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A detected technology with optional version and location context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technology {
    pub technology: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Confidence score 0.0 - 1.0 (not serialized in default output, used internally)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

impl Technology {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            technology: name.into(),
            version: None,
            location: None,
            confidence: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

/// Categories of technologies — mirrors W3Techs taxonomy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechCategory {
    #[serde(rename = "Content Management System")]
    ContentManagementSystem,
    #[serde(rename = "Server-side Programming Language")]
    ServerSideProgrammingLanguage,
    #[serde(rename = "Client-side Programming Language")]
    ClientSideProgrammingLanguage,
    #[serde(rename = "JavaScript Library")]
    JavaScriptLibrary,
    #[serde(rename = "JavaScript Framework")]
    JavaScriptFramework,
    #[serde(rename = "CSS Framework")]
    CssFramework,
    #[serde(rename = "Web Server")]
    WebServer,
    #[serde(rename = "Web Hosting Provider")]
    WebHostingProvider,
    #[serde(rename = "Data Center Provider")]
    DataCenterProvider,
    #[serde(rename = "Reverse Proxy Service")]
    ReverseProxyService,
    #[serde(rename = "DNS Server Provider")]
    DnsServerProvider,
    #[serde(rename = "SSL Certificate Authority")]
    SslCertificateAuthority,
    #[serde(rename = "Content Delivery Network")]
    ContentDeliveryNetwork,
    #[serde(rename = "JavaScript Content Delivery Network")]
    JavaScriptContentDeliveryNetwork,
    #[serde(rename = "Traffic Analysis Tool")]
    TrafficAnalysisTool,
    #[serde(rename = "Advertising Network")]
    AdvertisingNetwork,
    #[serde(rename = "Tag Manager")]
    TagManager,
    #[serde(rename = "Social Widget")]
    SocialWidget,
    #[serde(rename = "Site Element")]
    SiteElement,
    #[serde(rename = "Structured Data Format")]
    StructuredDataFormat,
    #[serde(rename = "Markup Language")]
    MarkupLanguage,
    #[serde(rename = "Character Encoding")]
    CharacterEncoding,
    #[serde(rename = "Image File Format")]
    ImageFileFormat,
    #[serde(rename = "Top Level Domain")]
    TopLevelDomain,
    #[serde(rename = "Server Location")]
    ServerLocation,
    #[serde(rename = "Content Language")]
    ContentLanguage,
    #[serde(rename = "Font Script")]
    FontScript,
    #[serde(rename = "Widget")]
    Widget,
    #[serde(rename = "PaaS")]
    PaaS,
    #[serde(rename = "Security")]
    Security,
    #[serde(rename = "E-commerce")]
    ECommerce,
    #[serde(rename = "Payment")]
    Payment,
    #[serde(rename = "Map")]
    Map,
    #[serde(rename = "Video Player")]
    VideoPlayer,
}

impl TechCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TechCategory::ContentManagementSystem => "Content Management System",
            TechCategory::ServerSideProgrammingLanguage => "Server-side Programming Language",
            TechCategory::ClientSideProgrammingLanguage => "Client-side Programming Language",
            TechCategory::JavaScriptLibrary => "JavaScript Library",
            TechCategory::JavaScriptFramework => "JavaScript Framework",
            TechCategory::CssFramework => "CSS Framework",
            TechCategory::WebServer => "Web Server",
            TechCategory::WebHostingProvider => "Web Hosting Provider",
            TechCategory::DataCenterProvider => "Data Center Provider",
            TechCategory::ReverseProxyService => "Reverse Proxy Service",
            TechCategory::DnsServerProvider => "DNS Server Provider",
            TechCategory::SslCertificateAuthority => "SSL Certificate Authority",
            TechCategory::ContentDeliveryNetwork => "Content Delivery Network",
            TechCategory::JavaScriptContentDeliveryNetwork => {
                "JavaScript Content Delivery Network"
            }
            TechCategory::TrafficAnalysisTool => "Traffic Analysis Tool",
            TechCategory::AdvertisingNetwork => "Advertising Network",
            TechCategory::TagManager => "Tag Manager",
            TechCategory::SocialWidget => "Social Widget",
            TechCategory::SiteElement => "Site Element",
            TechCategory::StructuredDataFormat => "Structured Data Format",
            TechCategory::MarkupLanguage => "Markup Language",
            TechCategory::CharacterEncoding => "Character Encoding",
            TechCategory::ImageFileFormat => "Image File Format",
            TechCategory::TopLevelDomain => "Top Level Domain",
            TechCategory::ServerLocation => "Server Location",
            TechCategory::ContentLanguage => "Content Language",
            TechCategory::FontScript => "Font Script",
            TechCategory::Widget => "Widget",
            TechCategory::PaaS => "PaaS",
            TechCategory::Security => "Security",
            TechCategory::ECommerce => "E-commerce",
            TechCategory::Payment => "Payment",
            TechCategory::Map => "Map",
            TechCategory::VideoPlayer => "Video Player",
        }
    }
}

/// Raw data collected from a site before analysis
#[derive(Debug, Default)]
pub struct SiteData {
    pub url: String,
    pub final_url: String,
    pub html: String,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<CookieInfo>,
    pub status_code: u16,
    pub http_version: HttpVersion,
    pub dns_records: DnsRecords,
    pub ip_address: Option<String>,
    pub tls_info: Option<TlsInfo>,
    pub response_time_ms: u64,
}

#[derive(Debug, Default, Clone)]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub http_only: bool,
    pub secure: bool,
    pub max_age_seconds: Option<i64>,
    pub same_site: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum HttpVersion {
    #[default]
    Http1,
    Http11,
    Http2,
    Http3,
}

impl HttpVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpVersion::Http1 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
            HttpVersion::Http3 => "HTTP/3",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DnsRecords {
    pub a_records: Vec<String>,
    pub aaaa_records: Vec<String>,
    pub mx_records: Vec<String>,
    pub ns_records: Vec<String>,
    pub txt_records: Vec<String>,
    pub cname_records: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TlsInfo {
    pub issuer_org: Option<String>,
    pub issuer_cn: Option<String>,
    pub subject_cn: Option<String>,
    pub not_after: Option<String>,
}

/// Final analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub domain: String,

    #[serde(flatten)]
    pub categories: HashMap<String, Vec<Technology>>,
}

impl AnalysisResult {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            categories: HashMap::new(),
        }
    }

    pub fn add(&mut self, category: TechCategory, tech: Technology) {
        self.categories
            .entry(category.as_str().to_string())
            .or_default()
            .push(tech);
    }

    pub fn extend(&mut self, category: TechCategory, techs: Vec<Technology>) {
        if !techs.is_empty() {
            self.categories
                .entry(category.as_str().to_string())
                .or_default()
                .extend(techs);
        }
    }

    /// Deduplicate technologies within each category
    pub fn dedup(&mut self) {
        for techs in self.categories.values_mut() {
            techs.dedup_by(|a, b| a.technology == b.technology);
        }
        // Remove empty categories
        self.categories.retain(|_, v| !v.is_empty());
    }
}
