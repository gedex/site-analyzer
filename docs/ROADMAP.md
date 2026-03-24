# Roadmap

## Recently Completed

### DNS-Based Detection ✅

Implemented DNS record pattern matching to detect infrastructure technologies:
- **DNS Providers** - Cloudflare, Amazon Route53
- **Email Providers** - Google Workspace (via MX and SPF records)
- **Record Types** - NS (nameservers), MX (mail servers), TXT (SPF, DKIM, DMARC), A, AAAA, CNAME
- **Example fingerprints:**
  - Cloudflare: `"dns": { "ns": "cloudflare\\.com$" }`
  - Google Workspace: `"dns": { "mx": "google\\.com$", "txt": "v=spf1.*include:_spf\\.google\\.com" }`
  - Amazon Route53: `"dns": { "ns": "awsdns.*\\.amazonaws\\.com$" }`

**Completed:** 2026-03-24

## Planned Features

### IP Geolocation

Detect server location by IP address:
- **Data Source** - ipinfo.io, MaxMind GeoIP, or similar
- **Information** - Country, city, region, hosting provider
- **Use Case** - Identify CDN usage, server locations

**Status:** Not started

### Batch Analysis Mode

Analyze multiple URLs from a file:
```bash
aw --url-list urls.txt --output results.json
```

**Features:**
- Concurrent analysis with configurable parallelism
- Rate limiting to avoid overloading targets
- Progress reporting
- Resume capability for interrupted runs
- Output formats: JSON, CSV, SQLite

**Status:** Not started

### Crawler + Database

Build a W3Techs-style analytics platform:

**Components:**
- Background crawler with scheduling
- PostgreSQL or SQLite storage
- Historical tracking (technology adoption over time)
- Trend analysis
- Technology popularity rankings

**Use Cases:**
- "What percentage of websites use React?"
- "Which CMS is growing fastest?"
- "Technology combinations (WordPress + WooCommerce)"

**Status:** Not started

### REST API Service

HTTP API for on-demand analysis:

```
GET /v1/analyze?url=example.com
GET /v1/technologies
GET /v1/categories
GET /v1/stats/technology/:name
```

**Features:**
- Axum-based HTTP server
- API authentication (optional)
- Rate limiting
- Response caching
- OpenAPI/Swagger documentation

**Status:** Not started

### Web UI

Interactive web interface:

**Pages:**
- Home: URL input + analyze
- Results: Technology breakdown with details
- Explorer: Browse technologies by category
- Statistics: Popular technologies, trends
- Compare: Side-by-side comparison of sites

**Tech Stack:**
- Frontend: React or Svelte
- Backend: REST API (above)
- Deployment: Docker container

**Status:** Not started

## Improvements

### Enhanced Detection

- **CSS Pattern Detection** - Currently minimal coverage
- **JavaScript Global Detection** - Better handling of edge cases
- **DOM Manipulation** - Detect dynamically loaded content
- **WebAssembly Detection** - Identify WASM frameworks

### Performance

- **Parallel Fingerprint Matching** - Use rayon for multi-threaded regex matching
- **Benchmark Suite** - Track performance over time
- **Memory Profiling** - Optimize allocations
- **Caching** - Cache DNS/TLS results for repeated analysis

### Developer Experience

- **Better Error Messages** - More actionable feedback
- **Debug Mode** - Show why each technology was/wasn't detected
- **Pattern Tester** - CLI tool to test regex patterns
- **Technology Templates** - Scaffolding for new fingerprints

## Technical Debt

- [ ] Implement IP geolocation
- [ ] Add metrics/instrumentation (tracing, Prometheus)
- [ ] Create benchmark suite
- [ ] Document internal APIs with rustdoc

## Contributing

See [Development Guide](DEVELOPMENT.md) for how to contribute to the roadmap.

Have an idea? Open an issue on GitHub to discuss!
