# Testing Guide

## Overview

Comprehensive test coverage with **50 tests** across all components.

```
✅ 50 tests total
✅ 100% pass rate
✅ All 1,272 technology fingerprints validated
```

## Running Tests

### All Tests

```bash
cargo test --package site-analyzer
```

### Specific Test Suite

```bash
# Detection tests (29 tests)
cargo test --test detection_test

# Fingerprint validation (5 tests)
cargo test --test fingerprints_test

# Pattern parsing (6 tests)
cargo test --test pattern_test

# Engine helpers (3 tests)
cargo test --test engine_test
```

### Single Test

```bash
cargo test test_detect_wordpress
```

### With Output

```bash
cargo test -- --nocapture
```

### With Logging

```bash
RUST_LOG=debug cargo test
```

## Test Suites

### 1. Detection Tests (35 tests)

**Location:** `crates/analyzer/tests/detection_test.rs`

Tests real-world technology detection with mock site data.

#### Pattern Type Coverage (11 tests)

Tests each detection mechanism:

- **HTML Pattern Detection** - Regex matching in HTML content
- **Script Src Detection** - Detecting via `<script src>` attributes
- **Meta Tag Detection** - Detecting via `<meta>` tags with version extraction
- **Header Detection** - Detecting via HTTP headers with version extraction
- **Cookie Detection** - Detecting via cookie names
- **TLS Cert Issuer Detection** - Detecting SSL/TLS certificate authorities
- **DNS NS Record Detection** - Detecting via nameserver records (e.g., Cloudflare)
- **DNS MX Record Detection** - Detecting via mail server records (e.g., Google Workspace)
- **DNS TXT Record Detection** - Detecting via SPF/DKIM records
- **DNS Route53 Detection** - Detecting AWS Route53 via NS patterns
- **Implies Chain** - Testing technology inference (WordPress → PHP + MySQL)

#### Popular Technology Detection (23 tests)

Tests detection of widely-used technologies:

**CMS Platforms:**
- WordPress (with version extraction)
- WordPress VIP (with header detection and implies)
- Automattic (with x-hacker header detection and implies)
- Drupal
- Joomla

**E-commerce:**
- Shopify
- WooCommerce
- Magento

**JavaScript Frameworks:**
- React
- Vue.js
- AngularJS
- Next.js
- Gatsby

**JavaScript Libraries:**
- jQuery (with version extraction)
- Bootstrap
- tailwindcss

**Web Servers:**
- Nginx (with version extraction)
- Apache (with version extraction)

**Runtime & Languages:**
- PHP (with version extraction)
- Express (Node.js)

**CDN & Services:**
- Cloudflare
- Google Analytics

#### Edge Cases (1 test)

- **Multiple Technologies** - Simultaneous detection of WordPress + jQuery + Bootstrap + Nginx + PHP
- **Version Extraction Accuracy** - Validates precise version capture

### 2. Fingerprint Validation Tests (5 tests)

**Location:** `crates/analyzer/tests/fingerprints_test.rs`

Validates all 1,270 technology JSON files:

- **All Technology Files Valid** - Comprehensive validation:
  - Valid JSON syntax
  - Required `cats` field present and non-empty
  - Category IDs in valid range (1-100)
  - Pattern fields are correct types (string or array)
  - Keyed patterns (headers, meta, cookies, js) are objects
  - All files present in `_mapping.json` (or auto-discovered)
  - No duplicate technology names
  - Regex patterns compile successfully

- **Mapping Has No Duplicates** - Ensures unique technology names

- **Generated Matches Individual Files** - Verifies `build.rs` merged correctly

- **Load Database** - Tests `FingerprintDb` loads successfully

- **WordPress Fingerprint** - Sample validation of specific technology

### 3. Pattern Parsing Tests (6 tests)

**Location:** `crates/analyzer/tests/pattern_test.rs`

Tests the Wappalyzer pattern format parser:

- **Simple Pattern** - Basic regex matching without version
- **Version Capture** - Extracts version from regex capture groups
- **Literal Version** - Hardcoded version in pattern
- **Confidence** - Confidence score parsing
- **Empty Pattern** - Presence-only checks (matches anything)
- **Bad Regex** - Graceful handling of invalid regex patterns

### 4. Engine Helper Tests (3 tests)

**Location:** `crates/analyzer/tests/engine_test.rs`

Tests core engine helper functions:

- **Extract Script Srcs** - Parses `<script src>` attributes from HTML
- **Extract Meta Tags** - Parses `<meta name/property>` tags from HTML
- **Database Loads** - Verifies fingerprint database initialization

### 5. Unit Tests (1 test)

**Location:** `crates/analyzer/src/fingerprints.rs`

- **Implied Tech Parsing** - Tests internal `parse_implied_tech()` helper

## Test Organization

```
crates/analyzer/
├── src/
│   └── fingerprints.rs          # 1 unit test
└── tests/
    ├── detection_test.rs        # 29 integration tests
    ├── fingerprints_test.rs     # 5 validation tests
    ├── pattern_test.rs          # 6 unit tests
    └── engine_test.rs           # 3 unit tests
```

## Writing Tests

### Detection Test Template

```rust
#[tokio::test]
async fn test_detect_my_framework() {
    let html = r#"
        <html>
        <head>
            <script src="https://cdn.example.com/myframework.min.js"></script>
        </head>
        </html>
    "#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let my_framework = find_tech(&result, "MyFramework");
    assert!(my_framework.is_some(), "MyFramework should be detected");
}
```

### With Headers

```rust
#[tokio::test]
async fn test_detect_with_headers() {
    let mut headers = HashMap::new();
    headers.insert("x-powered-by".to_string(), "Express".to_string());

    let data = make_test_site("", headers);
    let result = detect(&data).await;

    let express = find_tech(&result, "Express");
    assert!(express.is_some(), "Express should be detected");
}
```

### With Version Extraction

```rust
#[tokio::test]
async fn test_version_extraction() {
    let html = r#"<script src="/jquery.min.js?ver=3.7.1"></script>"#;
    let data = make_test_site(html, HashMap::new());
    let result = detect(&data).await;

    let jquery = find_tech(&result, "jQuery")
        .expect("jQuery should be detected");
    assert_eq!(jquery.version.as_deref(), Some("3.7.1"));
}
```

### With TLS Certificate

```rust
#[tokio::test]
async fn test_tls_detection() {
    let mut data = make_test_site("", HashMap::new());
    data.tls_info = Some(TlsInfo {
        issuer_org: Some("Let's Encrypt".to_string()),
        issuer_cn: Some("R3".to_string()),
        subject_cn: Some("example.com".to_string()),
        not_after: Some("2025-12-31".to_string()),
    });

    let result = detect(&data).await;

    let lets_encrypt = find_tech(&result, "Let's Encrypt");
    assert!(lets_encrypt.is_some());
}
```

## Test Coverage

| Component | Tests | Coverage | Status |
|-----------|-------|----------|--------|
| Detection Engine | 35 | All detection types | ✅ Complete |
| Fingerprint Files | 5 | All 1,272 files | ✅ Complete |
| Pattern Parser | 6 | All pattern types | ✅ Complete |
| Engine Helpers | 3 | HTML/Script parsing | ✅ Complete |
| Unit Tests | 1 | Internal helpers | ✅ Complete |

### What's Tested

✅ **Covered:**
- All detection mechanisms (HTML, scripts, headers, meta, cookies, TLS, DNS)
- DNS-based detection (NS, MX, TXT records)
- Technology inference chains (implies/excludes)
- Version extraction from all sources
- Pattern parsing (regex, confidence, version templates)
- All 1,272 technology JSON files structurally validated
- Popular technologies detection
- Edge cases (multiple simultaneous detections)

⚠️ **Not Covered (Future Work):**
- IP geolocation
- Live website testing (all tests use mock data)
- Performance/stress testing
- CSS pattern detection (less commonly used)
- JavaScript global detection edge cases

## Performance

```
Pattern tests:      ~0.01s  (fast)
Engine tests:       ~1.8s   (medium)
Fingerprint tests:  ~1.9s   (medium - validates all files)
Detection tests:    ~2.8s   (medium - 29 async tests)
─────────────────────────────────────────
Total:              ~6.5s
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run tests
        run: cargo test --package site-analyzer --verbose
```

## Troubleshooting

### Tests Fail After Adding Technology

**Issue:** New technology JSON is invalid

**Fix:**
```bash
# Validate JSON syntax
cat technologies/MyTech.json | jq .

# Run validation tests
cargo test --test fingerprints_test
```

### Detection Test Fails

**Issue:** Pattern doesn't match as expected

**Fix:**
```bash
# Check pattern syntax
# Test regex at https://regex101.com/

# Run with debug output
RUST_LOG=debug cargo test test_detect_my_framework -- --nocapture
```

### Build Script Error

**Issue:** `build.rs` fails to merge files

**Fix:**
```bash
# Check for syntax errors in JSON files
find technologies -name "*.json" -exec sh -c 'jq . "$1" > /dev/null || echo "Invalid: $1"' _ {} \;

# Rebuild from scratch
cargo clean
cargo build
```

## Maintenance

### When Adding a New Technology

1. Create JSON file in `technologies/`
2. Run `cargo test --test fingerprints_test` to validate
3. Optionally add detection test if it's a popular technology

### When Modifying Detection Logic

1. Run full test suite: `cargo test --package site-analyzer`
2. Add regression test if fixing a bug
3. Update this document if coverage changes

### Updating Test Data

When Wappalyzer patterns change:
1. Update individual JSON files
2. Run validation tests
3. Run detection tests
4. Update documentation if needed

## Best Practices

### Test Naming

- Use descriptive names: `test_detect_wordpress`
- Prefix with pattern type: `test_html_pattern_detection`
- Group related tests: `test_detect_*` for technology tests

### Test Structure

```rust
#[tokio::test]
async fn test_name() {
    // 1. Setup: Create test data
    let data = make_test_site(...);

    // 2. Execute: Run detection
    let result = detect(&data).await;

    // 3. Assert: Verify expectations
    assert!(find_tech(&result, "Tech").is_some());
}
```

### Assertions

- Use descriptive error messages
- Test positive cases (should detect)
- Test negative cases (should not detect)
- Verify version extraction when applicable

## Resources

- [Development Guide](DEVELOPMENT.md) - Adding technologies, workflow
- [Architecture](ARCHITECTURE.md) - System design, detection pipeline
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
