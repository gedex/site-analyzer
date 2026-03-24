use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    detectors::Detector,
    error::Result,
    types::{AnalysisResult, HttpVersion, SiteData, TechCategory, Technology},
};

static HTML5_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<!doctype html>").unwrap());
static XHTML_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)<!DOCTYPE html PUBLIC.*?XHTML ([\d.]+)"#).unwrap());
static CHARSET_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)charset=["']?([\w-]+)"#).unwrap());
static LANG_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)<html[^>]*\slang=["']([a-zA-Z-]+)["']"#).unwrap());

pub struct HtmlDetector;

#[async_trait]
impl Detector for HtmlDetector {
    fn name(&self) -> &'static str {
        "HtmlDetector"
    }

    async fn detect(&self, data: &SiteData, result: &mut AnalysisResult) -> Result<()> {
        let html = &data.html;
        let document = Html::parse_document(html);

        detect_markup(html, result);
        detect_encoding(html, &document, data, result);
        detect_structured_data(html, result);
        detect_site_elements(data, html, result);
        detect_image_formats(html, result);
        detect_tld_and_lang(data, html, result);

        Ok(())
    }
}

fn detect_markup(html: &str, result: &mut AnalysisResult) {
    // HTML5
    if HTML5_RE.is_match(&html[..html.len().min(500)]) {
        result.add(TechCategory::MarkupLanguage, Technology::new("HTML5"));
    }

    // XHTML
    if let Some(cap) = XHTML_RE.captures(&html[..html.len().min(500)]) {
        let xhtml_type = if html.contains("Transitional") {
            "XHTML Transitional"
        } else if html.contains("Strict") {
            "XHTML Strict"
        } else {
            "XHTML"
        };
        result.add(
            TechCategory::MarkupLanguage,
            Technology::new(xhtml_type).with_version(&cap[1]),
        );
    }
}

fn detect_encoding(
    html: &str,
    document: &Html,
    data: &SiteData,
    result: &mut AnalysisResult,
) {
    // From Content-Type header
    if let Some(ct) = data.headers.get("content-type") {
        if let Some(cap) = CHARSET_RE.captures(ct) {
            result.add(
                TechCategory::CharacterEncoding,
                Technology::new(cap[1].to_uppercase()),
            );
            return;
        }
    }

    // From meta charset
    if let Ok(sel) = Selector::parse("meta[charset]") {
        for el in document.select(&sel) {
            if let Some(charset) = el.value().attr("charset") {
                result.add(
                    TechCategory::CharacterEncoding,
                    Technology::new(charset.to_uppercase()),
                );
                return;
            }
        }
    }

    // From meta http-equiv
    if let Some(cap) = CHARSET_RE.captures(html) {
        result.add(
            TechCategory::CharacterEncoding,
            Technology::new(cap[1].to_uppercase()),
        );
    }
}

fn detect_structured_data(html: &str, result: &mut AnalysisResult) {
    // Open Graph
    if html.contains("og:title") || html.contains("property=\"og:") || html.contains("property='og:") {
        result.add(TechCategory::StructuredDataFormat, Technology::new("Open Graph"));
    }

    // Twitter/X Cards
    if html.contains("twitter:card") || html.contains("name=\"twitter:") {
        result.add(TechCategory::StructuredDataFormat, Technology::new("Twitter/X Cards"));
    }

    // JSON-LD
    if html.contains(r#"type="application/ld+json""#)
        || html.contains(r#"type='application/ld+json'"#)
    {
        result.add(TechCategory::StructuredDataFormat, Technology::new("JSON-LD"));
    }

    // Microdata
    if html.contains("itemscope") || html.contains("itemtype=") {
        result.add(TechCategory::StructuredDataFormat, Technology::new("Microdata"));
    }

    // RDFa
    if html.contains("vocab=") || html.contains("typeof=") || html.contains("property=\"schema:") {
        result.add(TechCategory::StructuredDataFormat, Technology::new("Generic RDFa"));
    }
}

fn detect_site_elements(data: &SiteData, html: &str, result: &mut AnalysisResult) {
    // CSS
    if html.contains("<link") && html.contains(".css") {
        result.add(TechCategory::SiteElement, Technology::new("External CSS"));
    }
    if html.contains("<style>") || html.contains("<style ") {
        result.add(TechCategory::SiteElement, Technology::new("Embedded CSS"));
    }
    if html.contains("style=") {
        result.add(TechCategory::SiteElement, Technology::new("Inline CSS"));
    }

    // Cookies
    if !data.cookies.is_empty() {
        // Session cookies (no max-age, no expires)
        let has_session = data.cookies.iter().any(|c| c.max_age_seconds.is_none());
        if has_session {
            result.add(TechCategory::SiteElement, Technology::new("Session Cookies"));
        }

        // Cookie expiry breakdown
        let has_hours = data.cookies.iter().any(|c| {
            c.max_age_seconds.map(|a| a > 0 && a <= 86400).unwrap_or(false)
        });
        let has_days = data.cookies.iter().any(|c| {
            c.max_age_seconds.map(|a| a > 86400 && a <= 86400 * 365).unwrap_or(false)
        });
        let has_years = data.cookies.iter().any(|c| {
            c.max_age_seconds.map(|a| a > 86400 * 365).unwrap_or(false)
        });

        if has_hours { result.add(TechCategory::SiteElement, Technology::new("Cookies expiring in hours")); }
        if has_days { result.add(TechCategory::SiteElement, Technology::new("Cookies expiring in days")); }
        if has_years { result.add(TechCategory::SiteElement, Technology::new("Cookies expiring in years")); }

        // HttpOnly / Secure
        if data.cookies.iter().any(|c| c.http_only) {
            result.add(TechCategory::SiteElement, Technology::new("HttpOnly Cookies"));
        }
        if data.cookies.iter().any(|c| !c.http_only) {
            result.add(TechCategory::SiteElement, Technology::new("Non-HttpOnly Cookies"));
        }
        if data.cookies.iter().any(|c| c.secure) {
            result.add(TechCategory::SiteElement, Technology::new("Secure Cookies"));
        }
        if data.cookies.iter().any(|c| !c.secure) {
            result.add(TechCategory::SiteElement, Technology::new("Non-Secure Cookies"));
        }
    }

    // Compression
    if let Some(enc) = data.headers.get("content-encoding") {
        if enc.contains("gzip") {
            result.add(TechCategory::SiteElement, Technology::new("Gzip Compression"));
        }
        if enc.contains("br") {
            result.add(TechCategory::SiteElement, Technology::new("Brotli Compression"));
        }
        if enc.contains("zstd") {
            result.add(TechCategory::SiteElement, Technology::new("Zstandard Compression"));
        }
    }

    // HTTP version
    match data.http_version {
        HttpVersion::Http2 => {
            result.add(TechCategory::SiteElement, Technology::new("HTTP/2"));
        }
        HttpVersion::Http3 => {
            result.add(TechCategory::SiteElement, Technology::new("HTTP/2"));
            result.add(TechCategory::SiteElement, Technology::new("HTTP/3"));
        }
        _ => {}
    }

    // Protocol / subdomain defaults
    if data.final_url.starts_with("https://") {
        result.add(TechCategory::SiteElement, Technology::new("Default protocol https"));
    }

    let parsed_url = Url::parse(&data.final_url).ok();
    if let Some(url) = parsed_url {
        if url.host_str().map(|h| h.starts_with("www.")).unwrap_or(false) {
            result.add(TechCategory::SiteElement, Technology::new("Default subdomain www"));
        }
    }

    // HSTS
    if data.headers.contains_key("strict-transport-security") {
        result.add(TechCategory::Security, Technology::new("HSTS"));
    }

    // Content Security Policy
    if data.headers.contains_key("content-security-policy") {
        result.add(TechCategory::Security, Technology::new("Content Security Policy"));
    }

    // X-Frame-Options
    if data.headers.contains_key("x-frame-options") {
        result.add(TechCategory::Security, Technology::new("X-Frame-Options"));
    }

    // Permissions-Policy / Feature-Policy
    if data.headers.contains_key("permissions-policy") || data.headers.contains_key("feature-policy") {
        result.add(TechCategory::Security, Technology::new("Permissions-Policy"));
    }
}

fn detect_image_formats(html: &str, result: &mut AnalysisResult) {
    let html_lower = html.to_lowercase();
    if html_lower.contains(".png") { result.add(TechCategory::ImageFileFormat, Technology::new("PNG")); }
    if html_lower.contains(".jpg") || html_lower.contains(".jpeg") {
        result.add(TechCategory::ImageFileFormat, Technology::new("JPEG"));
    }
    if html_lower.contains(".svg") { result.add(TechCategory::ImageFileFormat, Technology::new("SVG")); }
    if html_lower.contains(".webp") { result.add(TechCategory::ImageFileFormat, Technology::new("WebP")); }
    if html_lower.contains(".gif") { result.add(TechCategory::ImageFileFormat, Technology::new("GIF")); }
    if html_lower.contains(".avif") { result.add(TechCategory::ImageFileFormat, Technology::new("AVIF")); }
    if html_lower.contains(".ico") { result.add(TechCategory::ImageFileFormat, Technology::new("ICO")); }
}

fn detect_tld_and_lang(data: &SiteData, html: &str, result: &mut AnalysisResult) {
    // TLD
    if let Ok(url) = Url::parse(&data.final_url) {
        if let Some(host) = url.host_str() {
            if let Some(dot_pos) = host.rfind('.') {
                let tld = &host[dot_pos..];
                result.add(TechCategory::TopLevelDomain, Technology::new(tld));
            }
        }
    }

    // Content Language from header
    if let Some(lang) = data.headers.get("content-language") {
        result.add(TechCategory::ContentLanguage, Technology::new(lang_code_to_name(lang)));
        return;
    }

    // Content Language from html lang attribute
    if let Some(cap) = LANG_RE.captures(&html[..html.len().min(2000)]) {
        let code = &cap[1];
        result.add(
            TechCategory::ContentLanguage,
            Technology::new(lang_code_to_name(code)),
        );
    }
}

fn lang_code_to_name(code: &str) -> &'static str {
    // Normalize to just language part
    let lang = code.split('-').next().unwrap_or(code).to_lowercase();
    match lang.as_str() {
        "en" => "English",
        "de" => "German",
        "fr" => "French",
        "es" => "Spanish",
        "it" => "Italian",
        "pt" => "Portuguese",
        "nl" => "Dutch",
        "ru" => "Russian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "ar" => "Arabic",
        "pl" => "Polish",
        "sv" => "Swedish",
        "no" => "Norwegian",
        "da" => "Danish",
        "fi" => "Finnish",
        "tr" => "Turkish",
        "he" => "Hebrew",
        "vi" => "Vietnamese",
        "th" => "Thai",
        "id" => "Indonesian",
        "cs" => "Czech",
        "sk" => "Slovak",
        "hu" => "Hungarian",
        "ro" => "Romanian",
        "uk" => "Ukrainian",
        "bg" => "Bulgarian",
        "hr" => "Croatian",
        "el" => "Greek",
        "ca" => "Catalan",
        _ => "Unknown",
    }
}
