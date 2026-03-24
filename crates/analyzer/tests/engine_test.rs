use scraper::Html;
use site_analyzer::engine::{extract_meta_tags, extract_script_srcs};
use site_analyzer::FingerprintDb;

#[test]
fn test_extract_script_srcs() {
    let html = r#"
        <script src="/wp-includes/js/jquery.js?ver=3.6"></script>
        <script src='https://cdn.example.com/react.min.js'></script>
        <script>inline code</script>
    "#;
    let srcs = extract_script_srcs(html);
    assert_eq!(srcs.len(), 2);
    assert!(srcs[0].contains("jquery.js"));
    assert!(srcs[1].contains("react.min.js"));
}

#[test]
fn test_extract_meta_tags() {
    let html = r#"
        <html><head>
        <meta name="generator" content="WordPress 6.2">
        <meta property="og:title" content="My Site">
        </head></html>
    "#;
    let doc = Html::parse_document(html);
    let meta = extract_meta_tags(&doc);
    assert_eq!(
        meta.get("generator").map(|s| s.as_str()),
        Some("WordPress 6.2")
    );
    assert_eq!(meta.get("og:title").map(|s| s.as_str()), Some("My Site"));
}

#[test]
fn test_db_loads() {
    // Test that FingerprintDb loads successfully
    let db = FingerprintDb::load();
    assert!(db.fingerprints.len() > 100);
}
