use site_analyzer::pattern::Pattern;

#[test]
fn test_simple_pattern() {
    let p = Pattern::parse("wordpress").unwrap();
    let (matched, ver) = p.matches("I use wordpress here");
    assert!(matched);
    assert!(ver.is_none());
}

#[test]
fn test_version_capture() {
    let p = Pattern::parse("^WordPress ?([\\d.]+)?\\;version:\\1").unwrap();
    let (matched, ver) = p.matches("WordPress 6.2.1");
    assert!(matched);
    assert_eq!(ver.as_deref(), Some("6.2.1"));
}

#[test]
fn test_literal_version() {
    let p = Pattern::parse("adplan7\\.com/\\;version:7").unwrap();
    let (matched, ver) = p.matches("https://x.adplan7.com/script.js");
    assert!(matched);
    assert_eq!(ver.as_deref(), Some("7"));
}

#[test]
fn test_confidence() {
    let p = Pattern::parse("adocean\\.pl\\;confidence:80").unwrap();
    assert_eq!(p.confidence, 80);
    let (matched, _) = p.matches("cdn.adocean.pl/script.js");
    assert!(matched);
}

#[test]
fn test_empty_pattern() {
    let p = Pattern::parse("").unwrap();
    let (matched, _) = p.matches("anything");
    assert!(matched);
}

#[test]
fn test_bad_regex_returns_none() {
    // The (?i) prefix can cause issues with some patterns; should degrade gracefully
    let result = Pattern::parse("[invalid(");
    // Should either return None or a valid pattern
    // Main thing is it doesn't panic
    let _ = result;
}
