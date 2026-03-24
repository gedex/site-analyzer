# Architecture

## Overview

site-analyzer is a Rust-based web technology detection tool that analyzes websites to identify CMS platforms, JavaScript frameworks, web servers, CDNs, analytics tools, SSL certificates, and more.

## Key Concepts

### 1. Detection Pipeline

```
URL → Fetch → Probe (DNS/TLS) → Detect → Infer → Output
```

**Fetch** - Download HTML, headers, cookies
**Probe** - Optional DNS records and TLS certificate info
**Detect** - Run 1,270 fingerprints against collected data
**Infer** - Resolve technology chains (WordPress → PHP → MySQL)
**Output** - Categorized results (JSON or human-readable)

### 2. Fingerprint-Based Detection

Technologies are identified by matching patterns against:
- HTML content (regex patterns)
- Script `src` attributes
- HTTP response headers
- `<meta>` tags
- Cookies
- TLS certificate issuers
- DNS records (NS, MX, TXT)
- JavaScript globals (in inline scripts)

### 3. Wappalyzer Pattern Format

Patterns use the format:
```
regex_pattern\;version:\1\;confidence:75
```

- Before `\;` = regex pattern (case-insensitive)
- `version:\1` = version template (capture groups)
- `confidence:N` = 0-100 confidence score
- Empty pattern = presence check only

Example:
```json
{
  "meta": {
    "generator": "^WordPress ?([\\d.]+)?\\;version:\\1"
  }
}
```

## Project Structure

```
site-analyzer/
├── crates/
│   ├── analyzer/              # Core library (site-analyzer)
│   │   ├── src/
│   │   │   ├── lib.rs         # Public API, Analyzer struct
│   │   │   ├── types.rs       # Core types (SiteData, Technology, etc.)
│   │   │   ├── error.rs       # Error types
│   │   │   ├── fetcher.rs     # HTTP client (reqwest)
│   │   │   ├── dns.rs         # DNS resolution (hickory-resolver)
│   │   │   ├── tls.rs         # TLS certificate extraction
│   │   │   ├── pattern.rs     # Wappalyzer pattern parser
│   │   │   ├── fingerprints.rs # JSON loading & parsing
│   │   │   ├── engine.rs      # Core detection engine
│   │   │   ├── category_map.rs # Category ID → enum mapping
│   │   │   ├── fingerprints/
│   │   │   │   ├── categories.json
│   │   │   │   └── technologies/  # 1,270 individual JSON files
│   │   │   │       ├── WordPress.json
│   │   │   │       ├── React.json
│   │   │   │       └── ...
│   │   │   └── detectors/
│   │   │       ├── mod.rs     # Detector trait
│   │   │       └── html.rs    # Supplemental detectors
│   │   ├── build.rs           # Compile-time JSON merge
│   │   └── tests/             # Integration tests
│   └── cli/                   # Binary crate (aw command)
│       └── src/main.rs        # CLI interface (clap)
├── generated/                 # Build artifacts
│   └── technologies.json      # Merged at compile time
└── docs/                      # Documentation
```

## Data Flow (Per URL Analysis)

### 1. Fetch Phase

```rust
Fetcher::fetch(url) → SiteData
```

- HTTP GET with user agent
- Follow redirects (max 5)
- Collect headers (lowercase keys)
- Extract cookies from `Set-Cookie`
- Detect HTTP version (1.1, 2, 3)
- Read response body as HTML

**Output:** `SiteData` struct with all collected information

### 2. Probe Phase (Optional)

**DNS Resolution:**
```rust
DnsResolver::resolve(domain) → DnsRecords
```
- A records (IPv4 addresses)
- MX records (mail servers)
- NS records (nameservers)
- TXT records (SPF, DKIM, etc.)

**TLS Certificate:**
```rust
fetch_tls_info(domain) → TlsInfo
```
- Issuer organization (e.g., "Let's Encrypt")
- Issuer common name
- Subject common name
- Expiry date

### 3. Detection Phase

```rust
engine::run_engine(site_data, result) → ()
```

**Primary Detection (engine.rs):**

1. Load `FingerprintDb` (once, via `Lazy<>`)
2. For each of 1,272 fingerprints:
   - Match URL patterns
   - Match HTTP headers (case-insensitive keys)
   - Match HTML body
   - Match `<script src>` attributes
   - Match `<meta>` tags
   - Match cookies
   - Match CSS patterns
   - Match TLS cert issuer
   - Match DNS records (NS, MX, TXT, A, AAAA, CNAME)
   - Match JavaScript globals
3. Collect matches with version & confidence
4. Resolve `implies` chains (BFS traversal)
5. Apply confidence filter (default: 50%)

**Supplemental Detection (detectors/html.rs):**

Detects site elements not in Wappalyzer data:
- Cookie attributes (HttpOnly, Secure, SameSite)
- HTTP/2, HTTP/3 support
- Compression (Gzip, Brotli, Zstd)
- Security headers (HSTS, CSP, X-Frame-Options)
- Structured data (Open Graph, JSON-LD, Twitter Cards)
- Markup types (HTML5, XHTML)
- Character encoding
- Image formats
- Top-level domain
- Content language

### 4. Inference Phase

**Technology Chains:**

Technologies can imply others:
```json
{
  "implies": ["PHP", "MySQL"]
}
```

Example chain:
```
WordPress detected
  ↓ implies
PHP detected
  ↓ implies
MySQL detected
```

Implementation uses breadth-first search to avoid infinite loops.

### 5. Output Phase

```rust
AnalysisResult { domain, categories: HashMap<String, Vec<Technology>> }
```

Grouped by category:
```json
{
  "domain": "example.com",
  "Content Management System": [
    { "technology": "WordPress", "version": "6.4" }
  ],
  "Server-side Programming Language": [
    { "technology": "PHP" }
  ]
}
```

## Core Components

### Fingerprint Database

**Compile-Time Build:**
1. `build.rs` scans `technologies/*.json`
2. Merges into `generated/technologies.json`
3. Embedded via `include_str!()` in binary

**Runtime Loading:**
1. Parse JSON once at startup (via `Lazy<FingerprintDb>`)
2. Store in memory for all analyses
3. ~10MB memory footprint

**File Structure:**
```json
{
  "technologies": {
    "WordPress": {
      "cats": [1, 11],
      "meta": { "generator": "^WordPress" },
      "implies": ["PHP", "MySQL"]
    }
  },
  "categories": {
    "1": { "name": "CMS" }
  }
}
```

### Pattern Matching Engine

**Pattern Types:**

1. **Simple Patterns** - String or array of strings:
   ```json
   "html": "wordpress"
   "html": ["pattern1", "pattern2"]
   ```

2. **Keyed Patterns** - Key-value pairs:
   ```json
   "headers": {
     "Server": "nginx/([\\d.]+)\\;version:\\1"
   }
   ```

3. **Version Extraction:**
   - Capture groups: `\1`, `\2`, etc.
   - Templates: `"version:2.\\1.\\2"`
   - Literals: `"version:7"`

4. **Confidence Scores:**
   - Default: 100
   - Lower for ambiguous patterns
   - Threshold filter (default: 50)

### Category System

70 categories mapped to Rust enum:
```rust
pub enum TechCategory {
    CMS,                    // 1
    WebServer,              // 22
    JavaScriptLibrary,      // 59
    SslCertificateAuthority,// 70
    // ...
}
```

Category mapping in `category_map.rs`.

## Performance Characteristics

### Memory

- Binary size: ~5MB (release)
- Runtime memory: ~10-20MB per analysis
- Fingerprint DB: ~10MB in memory (once)

### Speed

- Fetch: 100-300ms (network-dependent)
- DNS probe: 50-150ms (if enabled)
- TLS probe: 50-100ms (if enabled)
- Detection: 10-50ms (CPU-bound)
- **Total: 200-600ms per site**

### Concurrency

- Async/await throughout (tokio runtime)
- Can analyze multiple sites concurrently
- Each analysis is independent
- No shared mutable state

## Dependencies

### Key Libraries

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client with compression |
| `scraper` | HTML parsing (meta tags, scripts) |
| `hickory-resolver` | Async DNS resolution |
| `tokio-rustls` | TLS handshake for certs |
| `regex` | Pattern matching |
| `once_cell` | Lazy singleton for fingerprint DB |
| `serde_json` | JSON parsing |
| `clap` | CLI argument parsing |
| `colored` | Terminal colors |
| `tracing` | Structured logging |

## Future Enhancements

### Recently Implemented

1. **DNS-Based Detection** ✅
   - Detects DNS providers (Cloudflare, Route53)
   - Detects email providers (Google Workspace, Microsoft 365)
   - Uses NS/MX/TXT records for pattern matching
   - Example fingerprints: Cloudflare (NS), Google Workspace (MX/TXT), Amazon Route53 (NS)

### Planned Features

1. **IP Geolocation**
   - Detect server location by IP
   - Use ipinfo.io or similar service

3. **Batch Mode**
   - `--url-list file.txt`
   - Concurrent analysis with rate limiting

4. **Crawler + Database**
   - Scheduled re-crawling
   - SQLite/PostgreSQL storage
   - Historical tracking

5. **REST API**
   - Axum-based HTTP server
   - `GET /v1/siteinfo/:domain`
   - `GET /v1/stats/:technology`

6. **Web UI**
   - Search and analyze interface
   - Technology statistics
   - Trending technologies

### Technical Debt

- [ ] Implement IP geolocation
- [ ] Add metrics/instrumentation
- [ ] Benchmark suite

## Design Decisions

### Why Individual JSON Files?

**Pros:**
- Easy to add/edit single technology
- Git-friendly diffs
- Clear ownership per file
- Reduces merge conflicts

**Cons:**
- More files to manage
- Need build.rs to merge

**Trade-off:** Better DX for contributors outweighs slight build complexity.

### Why Compile-Time Embedding?

**Pros:**
- Single binary distribution
- Fast startup (no disk I/O)
- Immutable fingerprints (security)

**Cons:**
- Larger binary size
- Must rebuild to update fingerprints

**Trade-off:** Deployment simplicity > dynamic updates.

### Why Rust?

**Requirements:**
- Fast (analyze hundreds of sites/second)
- Concurrent (async I/O)
- Reliable (no crashes)
- Small binary (easy deployment)

**Rust delivers:**
- Zero-cost abstractions
- Fearless concurrency
- Memory safety
- Excellent HTTP/async ecosystem

## References

- [Wappalyzer](https://github.com/wappalyzer/wappalyzer) - Fingerprint data source
- [BuiltWith](https://builtwith.com/) - Inspiration
- [W3Techs](https://w3techs.com/) - Inspiration
