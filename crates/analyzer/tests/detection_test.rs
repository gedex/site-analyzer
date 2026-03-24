use site_analyzer::types::{AnalysisResult, DnsRecords, HttpVersion, SiteData, TlsInfo};
use std::collections::HashMap;

/// Helper to create test SiteData
fn make_test_site(html: &str, headers: HashMap<String, String>) -> SiteData {
    SiteData {
        url: "https://example.com".to_string(),
        final_url: "https://example.com".to_string(),
        html: html.to_string(),
        headers,
        cookies: vec![],
        status_code: 200,
        http_version: HttpVersion::Http2,
        response_time_ms: 100,
        ip_address: None,
        dns_records: DnsRecords::default(),
        tls_info: None,
    }
}

/// Helper to run detection on SiteData
async fn detect(data: &SiteData) -> AnalysisResult {
    let mut result = AnalysisResult::new("example.com");
    site_analyzer::engine::run_engine(data, &mut result)
        .await
        .expect("Engine should run successfully");
    result
}

/// Helper to find a technology by name in AnalysisResult
fn find_tech<'a>(result: &'a AnalysisResult, name: &str) -> Option<&'a site_analyzer::Technology> {
    result
        .categories
        .values()
        .flat_map(|techs| techs.iter())
        .find(|t| t.technology == name)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Pattern Type Tests - Test each detection mechanism works
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_html_pattern_detection() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <link rel="stylesheet" href="/wp-content/themes/style.css">
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let wordpress = find_tech(&result, "WordPress");
    assert!(
        wordpress.is_some(),
        "WordPress should be detected via HTML pattern"
    );
}

#[tokio::test]
async fn test_script_src_detection() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <script src="https://code.jquery.com/jquery-3.6.0.min.js"></script>
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let jquery = find_tech(&result, "jQuery");
    assert!(jquery.is_some(), "jQuery should be detected via script src");
}

#[tokio::test]
async fn test_meta_tag_detection() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="generator" content="WordPress 6.4.2">
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let wordpress = find_tech(&result, "WordPress");
    assert!(
        wordpress.is_some(),
        "WordPress should be detected via meta generator tag"
    );
    if let Some(wp) = wordpress {
        assert_eq!(wp.version.as_deref(), Some("6.4.2"));
    }
}

#[tokio::test]
async fn test_header_detection() {
    let html = "";
    let mut headers = HashMap::new();
    headers.insert("server".to_string(), "nginx/1.21.0".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    let nginx = find_tech(&result, "Nginx");
    assert!(nginx.is_some(), "Nginx should be detected via Server header");
    if let Some(n) = nginx {
        assert_eq!(n.version.as_deref(), Some("1.21.0"));
    }
}

#[tokio::test]
async fn test_cookie_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add a PHPSESSID cookie
    data.cookies.push(site_analyzer::types::CookieInfo {
        name: "PHPSESSID".to_string(),
        value: "abc123".to_string(),
        http_only: true,
        secure: false,
        max_age_seconds: None,
        same_site: None,
        domain: None,
    });

    let result = detect(&data).await;

    let php = find_tech(&result, "PHP");
    assert!(php.is_some(), "PHP should be detected via PHPSESSID cookie");
}

#[tokio::test]
async fn test_implies_chain() {
    // WordPress implies PHP and MySQL
    let html = r#"<meta name="generator" content="WordPress 6.0">"#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let wordpress = find_tech(&result, "WordPress");
    let php = find_tech(&result, "PHP");
    let mysql = find_tech(&result, "MySQL");

    assert!(wordpress.is_some(), "WordPress should be detected");
    assert!(php.is_some(), "PHP should be implied by WordPress");
    assert!(mysql.is_some(), "MySQL should be implied by WordPress");
}

#[tokio::test]
async fn test_tls_cert_issuer_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add Let's Encrypt TLS info
    data.tls_info = Some(TlsInfo {
        issuer_org: Some("Let's Encrypt".to_string()),
        issuer_cn: Some("R3".to_string()),
        subject_cn: Some("example.com".to_string()),
        not_after: Some("2025-12-31".to_string()),
    });

    let result = detect(&data).await;

    let lets_encrypt = find_tech(&result, "Let's Encrypt");
    assert!(
        lets_encrypt.is_some(),
        "Let's Encrypt should be detected via TLS cert issuer"
    );
}

#[tokio::test]
async fn test_dns_ns_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add Cloudflare nameservers
    data.dns_records.ns_records = vec![
        "ns1.cloudflare.com".to_string(),
        "ns2.cloudflare.com".to_string(),
    ];

    let result = detect(&data).await;

    let cloudflare = find_tech(&result, "Cloudflare");
    assert!(
        cloudflare.is_some(),
        "Cloudflare should be detected via NS records"
    );
}

#[tokio::test]
async fn test_dns_mx_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add Google Workspace MX records
    data.dns_records.mx_records = vec![
        "aspmx.l.google.com".to_string(),
        "alt1.aspmx.l.google.com".to_string(),
    ];

    let result = detect(&data).await;

    let google_workspace = find_tech(&result, "Google Workspace");
    assert!(
        google_workspace.is_some(),
        "Google Workspace should be detected via MX records"
    );
}

#[tokio::test]
async fn test_dns_txt_spf_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add Google SPF TXT record
    data.dns_records.txt_records = vec![
        "v=spf1 include:_spf.google.com ~all".to_string(),
    ];

    let result = detect(&data).await;

    let google_workspace = find_tech(&result, "Google Workspace");
    assert!(
        google_workspace.is_some(),
        "Google Workspace should be detected via TXT SPF record"
    );
}

#[tokio::test]
async fn test_dns_route53_detection() {
    let html = "";
    let mut data = make_test_site(html, HashMap::new());

    // Add AWS Route53 nameservers
    data.dns_records.ns_records = vec![
        "ns-1234.awsdns-12.org.amazonaws.com".to_string(),
        "ns-5678.awsdns-34.com.amazonaws.com".to_string(),
    ];

    let result = detect(&data).await;

    let route53 = find_tech(&result, "Amazon Route53");
    assert!(
        route53.is_some(),
        "Amazon Route53 should be detected via NS records"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Popular Technology Detection Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_detect_wordpress() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="WordPress 6.4.2">
            <link rel="stylesheet" href="/wp-content/themes/twentytwentyfour/style.css">
            <script src="/wp-includes/js/jquery/jquery.min.js"></script>
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let wp = find_tech(&result, "WordPress").expect("WordPress should be detected");
    assert_eq!(wp.version.as_deref(), Some("6.4.2"));
}

#[tokio::test]
async fn test_detect_react() {
    let html = r#"
        <html>
        <head>
            <script src="https://unpkg.com/react@18.2.0/umd/react.production.min.js"></script>
        </head>
        <body>
            <div id="root" data-reactroot=""></div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let react = find_tech(&result, "React");
    assert!(react.is_some(), "React should be detected");
}

#[tokio::test]
async fn test_detect_jquery() {
    let html = r#"
        <script src="https://code.jquery.com/jquery-3.7.1.min.js"></script>
        <script>jQuery(document).ready(function() {});</script>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let jquery = find_tech(&result, "jQuery").expect("jQuery should be detected");
    assert_eq!(jquery.version.as_deref(), Some("3.7.1"));
}

#[tokio::test]
async fn test_detect_nginx() {
    let mut headers = HashMap::new();
    headers.insert("server".to_string(), "nginx/1.24.0".to_string());

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let nginx = find_tech(&result, "Nginx").expect("Nginx should be detected");
    assert_eq!(nginx.version.as_deref(), Some("1.24.0"));
}

#[tokio::test]
async fn test_detect_apache() {
    let mut headers = HashMap::new();
    headers.insert(
        "server".to_string(),
        "Apache/2.4.58 (Ubuntu)".to_string(),
    );

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let apache = find_tech(&result, "Apache").expect("Apache should be detected");
    assert_eq!(apache.version.as_deref(), Some("2.4.58"));
}

#[tokio::test]
async fn test_detect_php() {
    let mut headers = HashMap::new();
    headers.insert("x-powered-by".to_string(), "PHP/8.2.0".to_string());

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let php = find_tech(&result, "PHP").expect("PHP should be detected");
    assert_eq!(php.version.as_deref(), Some("8.2.0"));
}

#[tokio::test]
async fn test_detect_multiple_technologies() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="WordPress 6.4">
            <script src="https://code.jquery.com/jquery-3.7.1.min.js"></script>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css">
        </head>
        </html>
    "#;
    let mut headers = HashMap::new();
    headers.insert("server".to_string(), "nginx/1.24.0".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    assert!(find_tech(&result, "WordPress").is_some(), "WordPress should be detected");
    assert!(find_tech(&result, "jQuery").is_some(), "jQuery should be detected");
    assert!(find_tech(&result, "Bootstrap").is_some(), "Bootstrap should be detected");
    assert!(find_tech(&result, "Nginx").is_some(), "Nginx should be detected");
    assert!(find_tech(&result, "PHP").is_some(), "PHP should be implied by WordPress");
}

#[tokio::test]
async fn test_detect_cloudflare() {
    let mut headers = HashMap::new();
    headers.insert("server".to_string(), "cloudflare".to_string());
    headers.insert("cf-ray".to_string(), "123456789-LAX".to_string());

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let cloudflare = find_tech(&result, "Cloudflare");
    assert!(cloudflare.is_some(), "Cloudflare should be detected");
}

#[tokio::test]
async fn test_detect_google_analytics() {
    let html = r#"
        <script src="https://www.google-analytics.com/analytics.js"></script>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let ga = find_tech(&result, "Google Analytics");
    assert!(ga.is_some(), "Google Analytics should be detected");
}

#[tokio::test]
async fn test_detect_bootstrap() {
    let html = r#"
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css">
        <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let bootstrap = find_tech(&result, "Bootstrap");
    assert!(bootstrap.is_some(), "Bootstrap should be detected");
}

#[tokio::test]
async fn test_detect_nextjs() {
    let html = r#"
        <html>
        <head>
            <script>window.__NEXT_DATA__ = {};</script>
        </head>
        <body>
            <div id="__next"></div>
        </body>
        </html>
    "#;
    let mut headers = HashMap::new();
    headers.insert("x-powered-by".to_string(), "Next.js 14.0.0".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    let nextjs = find_tech(&result, "Next.js");
    assert!(nextjs.is_some(), "Next.js should be detected");
}

#[tokio::test]
async fn test_detect_vuejs() {
    let html = r#"
        <html>
        <head>
            <script src="https://cdn.jsdelivr.net/npm/vue@3.3.4/dist/vue.global.js"></script>
        </head>
        <body>
            <div id="app" data-v-app=""></div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let vue = find_tech(&result, "Vue.js");
    assert!(vue.is_some(), "Vue.js should be detected");
}

#[tokio::test]
async fn test_detect_tailwind() {
    let html = r#"
        <html>
        <head>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@3.3.0/dist/tailwindcss.min.css">
        </head>
        <body>
            <div class="flex items-center">Styled with Tailwind</div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let tailwind = find_tech(&result, "tailwindcss");
    assert!(tailwind.is_some(), "tailwindcss should be detected");
}

#[tokio::test]
async fn test_detect_express() {
    let mut headers = HashMap::new();
    headers.insert("x-powered-by".to_string(), "Express".to_string());

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let express = find_tech(&result, "Express");
    assert!(express.is_some(), "Express should be detected");
}

#[tokio::test]
async fn test_detect_shopify() {
    let html = r#"
        <html>
        <head>
            <link href="//cdn.shopify.com/s/files/1/1234/5678/t/3/assets/theme.css" rel="stylesheet">
        </head>
        </html>
    "#;
    let mut headers = HashMap::new();
    headers.insert("x-shopid".to_string(), "12345678".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    let shopify = find_tech(&result, "Shopify");
    assert!(shopify.is_some(), "Shopify should be detected");
}

#[tokio::test]
async fn test_detect_woocommerce() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="WooCommerce 8.5.0">
            <link rel="stylesheet" href="/wp-content/plugins/woocommerce/assets/css/woocommerce.css">
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let woo = find_tech(&result, "WooCommerce");
    assert!(woo.is_some(), "WooCommerce should be detected");
}

#[tokio::test]
async fn test_detect_wordpress_vip() {
    let html = r#"
        <html>
        <head>
            <link rel="stylesheet" href="/wp-content/themes/style.css">
        </head>
        </html>
    "#;
    let mut headers = HashMap::new();
    headers.insert("x-powered-by".to_string(), "WordPress VIP <https://wpvip.com>".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    let wp_vip = find_tech(&result, "WordPress VIP");
    assert!(wp_vip.is_some(), "WordPress VIP should be detected via x-powered-by header");

    // Should also detect WordPress via implies
    let wordpress = find_tech(&result, "WordPress");
    assert!(wordpress.is_some(), "WordPress should be implied by WordPress VIP");
}

#[tokio::test]
async fn test_detect_automattic() {
    let html = "";
    let mut headers = HashMap::new();
    headers.insert("x-hacker".to_string(), "Want root?  Visit join.a8c.com/hacker and mention this header.".to_string());

    let data = make_test_site(html, headers);
    let result = detect(&data).await;

    let automattic = find_tech(&result, "Automattic");
    assert!(automattic.is_some(), "Automattic should be detected via x-hacker header");

    // Should also detect WordPress via implies
    let wordpress = find_tech(&result, "WordPress");
    assert!(wordpress.is_some(), "WordPress should be implied by Automattic");
}

#[tokio::test]
async fn test_detect_drupal() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="Drupal 10 (https://www.drupal.org)">
            <script src="/core/misc/drupal.js"></script>
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let drupal = find_tech(&result, "Drupal");
    assert!(drupal.is_some(), "Drupal should be detected");
}

#[tokio::test]
async fn test_detect_joomla() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="Joomla! - Open Source Content Management">
            <script src="/media/jui/js/jquery.min.js"></script>
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let joomla = find_tech(&result, "Joomla");
    assert!(joomla.is_some(), "Joomla should be detected");
}

#[tokio::test]
async fn test_detect_magento() {
    let html = r#"
        <html>
        <head>
            <script type="text/x-magento-init">
            {
                "*": {
                    "Magento_Ui/js/core/app": {}
                }
            }
            </script>
        </head>
        <body>
            <div class="page-wrapper"></div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let magento = find_tech(&result, "Magento");
    assert!(magento.is_some(), "Magento should be detected");
}

#[tokio::test]
async fn test_detect_gatsby() {
    let html = r#"
        <html>
        <head>
            <meta name="generator" content="Gatsby 5.12.0">
            <style id="gatsby-inlined-css"></style>
        </head>
        <body>
            <div id="___gatsby"></div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let gatsby = find_tech(&result, "Gatsby");
    assert!(gatsby.is_some(), "Gatsby should be detected");
}

#[tokio::test]
async fn test_detect_angular() {
    let html = r#"
        <html>
        <head>
            <script src="https://ajax.googleapis.com/ajax/libs/angularjs/1.8.2/angular.min.js"></script>
        </head>
        <body ng-app="myApp">
            <div ng-controller="myController"></div>
        </body>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let angular = find_tech(&result, "AngularJS");
    assert!(angular.is_some(), "AngularJS should be detected");
}

#[tokio::test]
async fn test_version_extraction_accuracy() {
    let html = r#"
        <script src="/wp-includes/js/jquery/jquery.min.js?ver=3.7.1"></script>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let jquery = find_tech(&result, "jQuery").expect("jQuery should be detected");

    assert_eq!(
        jquery.version.as_deref(),
        Some("3.7.1"),
        "Version should be extracted correctly"
    );
}
