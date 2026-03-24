use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use tracing::debug;

use crate::{
    category_map::map_category,
    error::Result,
    fingerprints::{Fingerprint, FingerprintDb},
    types::{AnalysisResult, SiteData, TechCategory, Technology},
};

/// Global singleton — load once, reuse across all analyses
static DB: Lazy<FingerprintDb> = Lazy::new(FingerprintDb::load);

/// A match produced by running a fingerprint against site data
#[derive(Debug)]
struct Match {
    tech_name: String,
    version: Option<String>,
    confidence: u8,
    cats: Vec<u32>,
}

/// Run the full Wappalyzer fingerprint database against `site_data` and
/// populate `result` with detected technologies.
pub async fn run_engine(data: &SiteData, result: &mut AnalysisResult) -> Result<()> {
    // Pre-parse HTML once
    let document = Html::parse_document(&data.html);
    let meta_map = extract_meta_tags(&document);
    let script_srcs = extract_script_srcs(&data.html);

    let mut matches: Vec<Match> = Vec::new();
    let mut matched_names: HashSet<String> = HashSet::new();

    for fp in &DB.fingerprints {
        if let Some(m) = match_fingerprint(fp, data, &meta_map, &script_srcs) {
            matched_names.insert(fp.name.clone());
            matches.push(m);
        }
    }

    // Resolve implies — add implied technologies that weren't directly detected
    let implied = resolve_implies(&matches, &matched_names);
    matches.extend(implied);

    // Commit all matches to result
    for m in &matches {
        commit_match(m, result);
    }

    debug!(
        "Engine matched {} technologies for {}",
        matches.len(),
        data.url
    );

    Ok(())
}

/// Try to match a single fingerprint against all signals in SiteData.
/// Returns the first/best match found (highest confidence).
fn match_fingerprint(
    fp: &Fingerprint,
    data: &SiteData,
    meta_map: &HashMap<String, String>,
    script_srcs: &[String],
) -> Option<Match> {
    let mut best_version: Option<String> = None;
    let mut best_confidence: u8 = 0;
    let mut found = false;

    // ── URL ──
    for pat in &fp.url_patterns {
        let (matched, ver) = pat.matches(&data.url);
        if matched {
            found = true;
            update_best(&mut best_confidence, &mut best_version, pat.confidence, ver);
        }
    }

    // ── HTTP Headers ──
    for kp in &fp.header_patterns {
        if let Some(header_val) = data.headers.get(&kp.key) {
            let (matched, ver) = kp.pattern.matches(header_val);
            if matched {
                found = true;
                update_best(
                    &mut best_confidence,
                    &mut best_version,
                    kp.pattern.confidence,
                    ver,
                );
            }
        }
    }

    // ── HTML body ──
    for pat in &fp.html_patterns {
        let (matched, ver) = pat.matches(&data.html);
        if matched {
            found = true;
            update_best(&mut best_confidence, &mut best_version, pat.confidence, ver);
        }
    }

    // ── Script src attributes ──
    for pat in &fp.script_patterns {
        for src in script_srcs {
            let (matched, ver) = pat.matches(src);
            if matched {
                found = true;
                update_best(&mut best_confidence, &mut best_version, pat.confidence, ver);
                break;
            }
        }
    }

    // ── Meta tags ──
    for kp in &fp.meta_patterns {
        if let Some(meta_val) = meta_map.get(&kp.key) {
            let (matched, ver) = kp.pattern.matches(meta_val);
            if matched {
                found = true;
                update_best(
                    &mut best_confidence,
                    &mut best_version,
                    kp.pattern.confidence,
                    ver,
                );
            }
        }
    }

    // ── Cookies ──
    for kp in &fp.cookie_patterns {
        // Match against cookie name (and value if available)
        for cookie in &data.cookies {
            let cookie_name_lower = cookie.name.to_lowercase();
            if cookie_name_lower == kp.key {
                let (matched, ver) = kp.pattern.matches(&cookie.value);
                if matched {
                    found = true;
                    update_best(
                        &mut best_confidence,
                        &mut best_version,
                        kp.pattern.confidence,
                        ver,
                    );
                }
            }
        }
    }

    // ── CSS patterns (matched against HTML body for simplicity) ──
    for pat in &fp.css_patterns {
        let (matched, ver) = pat.matches(&data.html);
        if matched {
            found = true;
            update_best(&mut best_confidence, &mut best_version, pat.confidence, ver);
        }
    }

    // ── certIssuer ──
    if let Some(issuer_pattern) = &fp.cert_issuer {
        if let Some(tls) = &data.tls_info {
            let issuer_str = format!(
                "{} {}",
                tls.issuer_org.as_deref().unwrap_or(""),
                tls.issuer_cn.as_deref().unwrap_or("")
            );
            if issuer_str
                .to_lowercase()
                .contains(&issuer_pattern.to_lowercase())
            {
                found = true;
                update_best(&mut best_confidence, &mut best_version, 100, None);
            }
        }
    }

    // ── JS global variable patterns ──
    // We can't execute JS, but we check for the variable name appearing as
    // a likely global declaration or assignment in inline scripts
    for kp in &fp.js_patterns {
        // e.g. "wp_username" → look for `wp_username` in HTML/scripts
        let js_name = kp.key.replace('.', "\\."); // escape dots for property paths
        let search_pat = format!("(?:^|[\\s;{{,=(]){}(?:[\\s;}},.=(]|$)", js_name);
        if let Ok(re) = regex::Regex::new(&search_pat) {
            if let Some(caps) = re.captures(&data.html) {
                let (matched, ver) = kp.pattern.matches(caps.get(0).map(|m| m.as_str()).unwrap_or(""));
                if matched || re.is_match(&data.html) {
                    found = true;
                    update_best(
                        &mut best_confidence,
                        &mut best_version,
                        kp.pattern.confidence,
                        ver,
                    );
                }
            }
        }
    }

    // ── DNS records ──
    for kp in &fp.dns_patterns {
        let dns_type = kp.key.to_lowercase();
        let records = match dns_type.as_str() {
            "ns" => &data.dns_records.ns_records,
            "mx" => &data.dns_records.mx_records,
            "txt" => &data.dns_records.txt_records,
            "a" => &data.dns_records.a_records,
            "aaaa" => &data.dns_records.aaaa_records,
            "cname" => &data.dns_records.cname_records,
            _ => continue,
        };

        for record in records {
            let (matched, ver) = kp.pattern.matches(record);
            if matched {
                found = true;
                update_best(
                    &mut best_confidence,
                    &mut best_version,
                    kp.pattern.confidence,
                    ver,
                );
                break;
            }
        }
    }

    if found {
        Some(Match {
            tech_name: fp.name.clone(),
            version: best_version.filter(|v| !v.is_empty()),
            confidence: best_confidence,
            cats: fp.cats.clone(),
        })
    } else {
        None
    }
}

/// Update best confidence + version if the new match is better
fn update_best(
    best_confidence: &mut u8,
    best_version: &mut Option<String>,
    confidence: u8,
    version: Option<String>,
) {
    if confidence >= *best_confidence {
        *best_confidence = confidence;
        if version.is_some() && best_version.is_none() {
            *best_version = version;
        }
    }
}

/// Walk the `implies` chains for all matched techs and emit additional matches
fn resolve_implies(
    matches: &[Match],
    already_matched: &HashSet<String>,
) -> Vec<Match> {
    let mut result: Vec<Match> = Vec::new();
    let mut seen: HashSet<String> = already_matched.clone();

    // Collect all implies from the current match set
    let mut queue: Vec<(String, Option<String>, Vec<u32>)> = Vec::new();

    for m in matches {
        if let Some(fp) = DB.fingerprints.iter().find(|f| f.name == m.tech_name) {
            for implied in &fp.implies {
                if !seen.contains(&implied.name) {
                    // Look up the implied tech's categories
                    let cats = DB
                        .fingerprints
                        .iter()
                        .find(|f| f.name == implied.name)
                        .map(|f| f.cats.clone())
                        .unwrap_or_default();
                    queue.push((implied.name.clone(), implied.version.clone(), cats));
                }
            }
        }
    }

    // BFS — also follow implies-of-implies
    while let Some((name, version, cats)) = queue.pop() {
        if seen.contains(&name) {
            continue;
        }
        seen.insert(name.clone());

        result.push(Match {
            tech_name: name.clone(),
            version,
            confidence: 100,
            cats: cats.clone(),
        });

        // Also follow this implied tech's own implies
        if let Some(fp) = DB.fingerprints.iter().find(|f| f.name == name) {
            for nested in &fp.implies {
                if !seen.contains(&nested.name) {
                    let nested_cats = DB
                        .fingerprints
                        .iter()
                        .find(|f| f.name == nested.name)
                        .map(|f| f.cats.clone())
                        .unwrap_or_default();
                    queue.push((nested.name.clone(), nested.version.clone(), nested_cats));
                }
            }
        }
    }

    result
}

/// Add a match to the AnalysisResult, mapping category IDs to TechCategory
fn commit_match(m: &Match, result: &mut AnalysisResult) {
    let mut tech = Technology::new(&m.tech_name);
    if let Some(v) = &m.version {
        tech = tech.with_version(v);
    }
    if m.confidence < 100 {
        tech = tech.with_confidence(m.confidence as f32 / 100.0);
    }

    // Use the first recognised category from cats list; fall back to "Miscellaneous"
    let category = m
        .cats
        .iter()
        .find_map(|&id| map_category(id))
        .unwrap_or(TechCategory::Widget);

    result.add(category, tech);
}

// ── HTML helpers ─────────────────────────────────────────────────────────────

/// Extract all <meta> tag name/property → content mappings (lowercased keys)
pub fn extract_meta_tags(document: &Html) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Ok(sel) = Selector::parse("meta") {
        for el in document.select(&sel) {
            let content = el
                .value()
                .attr("content")
                .unwrap_or("")
                .to_string();

            // name= attribute
            if let Some(name) = el.value().attr("name") {
                map.insert(name.to_lowercase(), content.clone());
            }

            // property= attribute (Open Graph etc.)
            if let Some(prop) = el.value().attr("property") {
                map.insert(prop.to_lowercase(), content.clone());
            }

            // http-equiv=
            if let Some(equiv) = el.value().attr("http-equiv") {
                map.insert(equiv.to_lowercase(), content.clone());
            }
        }
    }

    map
}

/// Extract all <script src="..."> values from raw HTML
pub fn extract_script_srcs(html: &str) -> Vec<String> {
    let mut srcs = Vec::new();

    // Quick scan — no need for full DOM parse since we only want src= values
    let mut pos = 0;
    let html_lower = html.to_lowercase();

    while let Some(script_pos) = html_lower[pos..].find("<script") {
        let abs = pos + script_pos;
        let end = html_lower[abs..].find('>').map(|e| abs + e).unwrap_or(html.len());
        let tag_slice = &html[abs..end];

        // Extract src="..." or src='...'
        if let Some(src) = extract_attr_value(tag_slice, "src") {
            srcs.push(src);
        }

        pos = end + 1;
        if pos >= html.len() {
            break;
        }
    }

    srcs
}

/// Pull an attribute value out of a raw HTML tag fragment
fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
    let search = format!("{}=", attr);
    let tag_lower = tag.to_lowercase();
    let pos = tag_lower.find(&search)?;
    let rest = &tag[pos + search.len()..];

    if rest.starts_with('"') {
        let end = rest[1..].find('"')?;
        Some(rest[1..1 + end].to_string())
    } else if rest.starts_with('\'') {
        let end = rest[1..].find('\'')?;
        Some(rest[1..1 + end].to_string())
    } else {
        // Unquoted attribute
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

