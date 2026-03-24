/// Wappalyzer patterns look like:
///   "WordPress"
///   "^WordPress ?([\\d.]+)?\\;version:\\1"
///   "adocean\\.pl\\;confidence:80"
///   "^(.+)$\\;version:\\1\\;confidence:75"
///
/// The `\;` separator splits the regex from metadata tags.
/// Tags: `version:<template>`, `confidence:<0-100>`
///
/// Version templates use `\1`, `\2` etc. for regex capture group references.
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Pattern {
    /// Compiled regex (from the part before any `\;`)
    pub regex: Regex,

    /// Raw version template, e.g. `"\\1"` or `"2.\\1"` or `"7"` (literal)
    pub version_template: Option<String>,

    /// Confidence 0–100, default 100
    pub confidence: u8,
}

impl Pattern {
    /// Parse a raw Wappalyzer pattern string into a `Pattern`.
    /// Returns `None` if the regex fails to compile (we skip bad patterns silently).
    pub fn parse(raw: &str) -> Option<Self> {
        // Split on `\;` — the Wappalyzer separator
        let parts: Vec<&str> = raw.split("\\;").collect();
        let regex_src = parts[0];

        let mut version_template: Option<String> = None;
        let mut confidence: u8 = 100;

        for tag in parts.iter().skip(1) {
            if let Some(v) = tag.strip_prefix("version:") {
                if !v.is_empty() {
                    version_template = Some(v.to_string());
                }
            } else if let Some(c) = tag.strip_prefix("confidence:") {
                confidence = c.parse::<u8>().unwrap_or(100);
            }
        }

        // Empty pattern = always matches (presence check, no regex needed)
        // We use `.*` so the compiled regex always succeeds on non-empty input.
        let effective_src = if regex_src.is_empty() { ".*" } else { regex_src };

        // Build case-insensitive regex; fall back gracefully on bad patterns
        let regex = match Regex::new(&format!("(?i){}", effective_src)) {
            Ok(r) => r,
            Err(_) => {
                // Try without the (?i) prefix in case the pattern itself has flags
                match Regex::new(effective_src) {
                    Ok(r) => r,
                    Err(_) => return None,
                }
            }
        };

        Some(Pattern {
            regex,
            version_template,
            confidence,
        })
    }

    /// Try to match `text`. Returns `(matched, version_string)`.
    pub fn matches(&self, text: &str) -> (bool, Option<String>) {
        match self.regex.captures(text) {
            None => (false, None),
            Some(caps) => {
                let version = self
                    .version_template
                    .as_ref()
                    .map(|tmpl| resolve_version_template(tmpl, &caps));
                (true, version)
            }
        }
    }
}

/// Resolve a version template like `"\\1"`, `"2.\\1.\\2"`, or `"7"` using
/// the capture groups from a successful match.
fn resolve_version_template(tmpl: &str, caps: &regex::Captures) -> String {
    let mut result = tmpl.to_string();

    // Replace \1, \2, ... \9 with the corresponding capture group text
    for i in (1..=9).rev() {
        let placeholder = format!("\\{}", i);
        if result.contains(&placeholder) {
            let replacement = caps
                .get(i)
                .map(|m| m.as_str())
                .unwrap_or("");
            result = result.replace(&placeholder, replacement);
        }
    }

    // Trim whitespace and trailing dots/slashes that show up when a group
    // didn't match (e.g. optional version group)
    result.trim().trim_end_matches('.').trim().to_string()
}

/// A field-value pattern pair used for headers/meta/cookies/js
#[derive(Debug, Clone)]
pub struct KeyedPattern {
    /// The key (header name, meta name, cookie name, JS property)
    pub key: String,
    /// The pattern to match against the value
    pub pattern: Pattern,
}

/// Parse Wappalyzer's flexible "string or array of strings" into a Vec<Pattern>
pub fn parse_patterns_list(value: &serde_json::Value) -> Vec<Pattern> {
    match value {
        serde_json::Value::String(s) => Pattern::parse(s).into_iter().collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().and_then(Pattern::parse))
            .collect(),
        _ => vec![],
    }
}

/// Parse Wappalyzer's flexible "string or array of strings" for implies/excludes
pub fn parse_string_list(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}
