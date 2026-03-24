# Development Guide

## Quick Start

### Contributing

To add a new technology, create a JSON file in `crates/analyzer/src/fingerprints/technologies/`:

```bash
cat > crates/analyzer/src/fingerprints/technologies/MyFramework.json << 'EOF'
{
  "cats": [18],
  "website": "https://myframework.com",
  "html": "<div data-framework=\"myframework\">",
  "scripts": "myframework\\.min\\.js\\?v=([\\d.]+)\\;version:\\1",
  "implies": ["Node.js"]
}
EOF

cargo build
cargo test
```

See [Adding a New Technology](#adding-a-new-technology) for details.

### Running Tests

```bash
# Run all tests
cargo test --package site-analyzer

# Run specific test suite
cargo test --test detection_test

# With verbose output
cargo test -- --nocapture
```

See [Testing](#testing) section for details.

## Getting Started

### Prerequisites

- Rust 1.83+ (run `rustup update stable`)
- Git (optional)

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test --package site-analyzer

# Run CLI
cargo run --bin aw -- example.com
```

## Project Workflow

### Adding a New Technology

**1. Create JSON file:**

```bash
cd crates/analyzer/src/fingerprints/technologies

cat > "MyFramework.json" << 'EOF'
{
  "cats": [18],
  "website": "https://myframework.com",
  "description": "A modern web framework",
  "headers": {
    "X-Powered-By": "MyFramework"
  },
  "html": "<div[^>]+data-framework=\"myframework\"",
  "scripts": "myframework\\.min\\.js\\?v=([\\d.]+)\\;version:\\1",
  "implies": ["Node.js"]
}
EOF
```

**2. Build:**

```bash
cargo build
```

That's it! The technology is automatically discovered and included.

**3. (Optional) Add mapping if filename ≠ tech name:**

If you want "MyFramework" to display as "My Framework", edit `_mapping.json`:

```json
{
  "MyFramework": "My Framework"
}
```

### Editing a Technology

```bash
vim crates/analyzer/src/fingerprints/technologies/WordPress.json
cargo build
```

### Removing a Technology

```bash
rm crates/analyzer/src/fingerprints/technologies/OldTech.json
cargo build
```

## Technology JSON Format

### Required Fields

```json
{
  "cats": [18, 22]  // Array of category IDs (required)
}
```

### Optional Detection Fields

```json
{
  "html": "pattern",              // HTML content regex
  "scripts": ["pattern1"],        // <script src> patterns
  "headers": {                    // HTTP headers
    "Server": "nginx"
  },
  "meta": {                       // <meta> tags
    "generator": "^WordPress"
  },
  "cookies": {                    // Cookie names
    "PHPSESSID": ""
  },
  "js": {                         // JavaScript globals
    "__NEXT_DATA__": ""
  },
  "dns": {                        // DNS records
    "ns": "cloudflare\\.com$",
    "mx": "google\\.com$",
    "txt": "v=spf1.*include:_spf\\.google\\.com"
  },
  "url": "pattern",               // URL pattern
  "css": "pattern",               // CSS in HTML
  "certIssuer": "Let's Encrypt"   // TLS cert issuer
}
```

### Pattern Format

Patterns use Wappalyzer format:
```
regex_pattern\;version:\1\;confidence:75
```

**Examples:**

```json
{
  // Simple presence check
  "html": "wordpress",

  // With version extraction
  "scripts": "jquery-([\\d.]+)\\.min\\.js\\;version:\\1",

  // With confidence score
  "html": "some-pattern\\;confidence:50",

  // Multiple patterns (OR logic)
  "html": ["pattern1", "pattern2"]
}
```

### Metadata Fields

```json
{
  "website": "https://example.com",     // Homepage
  "description": "Short description",    // What it is
  "icon": "Technology.svg",              // Icon filename
  "cpe": "cpe:/a:vendor:product"         // Common Platform Enumeration
}
```

### Technology Relationships

```json
{
  "implies": ["PHP", "MySQL"],          // Technologies this implies
  "excludes": ["Apache"]                 // Mutually exclusive with
}
```

**Implies Example:**
```
WordPress detected → implies PHP → implies MySQL
```

### Category IDs

Common categories:

| ID | Category |
|----|----------|
| 1 | CMS |
| 6 | Ecommerce |
| 10 | Analytics |
| 12 | JavaScript frameworks |
| 18 | Web frameworks |
| 22 | Web servers |
| 27 | Programming languages |
| 31 | CDN |
| 59 | JavaScript libraries |
| 70 | SSL/TLS Certificate authorities |

See `categories.json` for the complete list of 70 categories.

## How Build Works

### Compile-Time Merge

```
technologies/WordPress.json
technologies/React.json       } build.rs scans directory
technologies/...              } merges into single JSON
                              ↓
generated/technologies.json   ← merged file
                              ↓
include_str!("...")           ← embedded in binary
```

**build.rs logic:**
1. Scan `technologies/*.json` (skips `_mapping.json`)
2. For each file:
   - Use `_mapping.json` for tech name if exists
   - Otherwise use filename as tech name
3. Merge all into single JSON structure
4. Write to `generated/technologies.json`
5. Gets embedded via `include_str!()` in `fingerprints.rs`

### Automatic Discovery

No scripts needed! `build.rs` automatically:
- Discovers new files
- Uses filename as default name
- Falls back to `_mapping.json` for custom names

## Testing

### Run All Tests

```bash
cargo test --package site-analyzer
```

### Test Categories

```bash
# Detection tests (29 tests - popular technologies)
cargo test --test detection_test

# Fingerprint validation (5 tests - all 1,270 files)
cargo test --test fingerprints_test

# Pattern parsing (6 tests)
cargo test --test pattern_test

# Engine helpers (3 tests)
cargo test --test engine_test
```

### Adding a Detection Test

Edit `crates/analyzer/tests/detection_test.rs`:

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

### Writing Good Tests

**Test pattern types:**
- HTML patterns
- Script src
- Headers
- Meta tags
- Cookies
- TLS certs

**Test popular technologies:**
- Add tests for widely-used technologies
- Include version extraction tests
- Test technology chains (implies)

See [Testing Guide](TESTING.md) for more details.

## Code Style

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run Clippy
cargo clippy --all-targets --all-features

# Fix warnings automatically
cargo clippy --fix
```

### Naming Conventions

- **Files:** `snake_case.rs`
- **Functions:** `snake_case()`
- **Types:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case`

## Troubleshooting

### Build Fails with "Failed to read _mapping.json"

The mapping file is optional now. If you see this error, your `build.rs` might be outdated. Make sure it uses:

```rust
let mapping: HashMap<String, String> = if mapping_file.exists() {
    // load mapping
} else {
    HashMap::new()
};
```

### Technology Not Detected

**Check:**
1. Pattern syntax is correct (escape special regex chars)
2. Pattern matches your test HTML
3. Category ID is valid (1-70)
4. File was included in build (check `cargo build` output)

**Debug:**
```bash
# Run with verbose logging
cargo run --bin aw -- example.com -v

# Check generated file
cat generated/technologies.json | grep "MyTech"
```

### Tests Fail After Adding Technology

**Common issues:**
1. Invalid JSON syntax
2. Missing required `cats` field
3. Invalid regex pattern
4. Duplicate technology name

**Fix:**
```bash
# Validate JSON
cat technologies/MyTech.json | jq .

# Run validation tests
cargo test --test fingerprints_test
```

## Project History

### Recent Changes

**Technology Management Simplification:**
- Removed monolithic `technologies.json` (371KB)
- Removed `split_technologies.py` script
- Made `_mapping.json` optional
- Auto-discovery in `build.rs`

**Result:** Simplified from 4 steps to 2 steps when adding technologies.

**Before:**
1. Create file
2. Run `python3 scripts/split_technologies.py --remap-only`
3. Edit `_mapping.json`
4. `cargo build`

**After:**
1. Create file
2. `cargo build`

## Contributing Guidelines

### Before Submitting

1. **Test:** Run `cargo test --package site-analyzer`
2. **Format:** Run `cargo fmt`
3. **Lint:** Run `cargo clippy`
4. **Validate:** Ensure technology JSON is valid
5. **Document:** Update docs if needed

### Pull Request Checklist

- [ ] Tests pass
- [ ] Code formatted (`cargo fmt`)
- [ ] No Clippy warnings
- [ ] Technology JSON validated
- [ ] Detection test added (if new tech)
- [ ] Documentation updated (if needed)

## Resources

- [Architecture](ARCHITECTURE.md) - System design and data flow
- [Testing](TESTING.md) - Test coverage and running tests
- [Releasing](RELEASING.md) - Build and release process
- [Roadmap](ROADMAP.md) - Planned features
- [Wappalyzer Docs](https://www.wappalyzer.com/docs) - Pattern format reference
