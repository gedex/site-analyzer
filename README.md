# site-analyzer

A Rust-based tool that analyzes websites to detect which technologies they use — CMS, JavaScript frameworks, web servers, CDNs, analytics tools, SSL certificates, and more.

## Installation

### Shell installer (macOS, Linux, Windows)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gedex/site-analyzer/releases/latest/download/aw-installer.sh | sh
```

### Homebrew
```bash
brew install gedex/tap/aw
```

### Pre-built binaries
Download from [releases](https://github.com/gedex/site-analyzer/releases/latest)

### Cargo
```bash
cargo install aw
```

### Build from source
```bash
cargo build --release
./target/release/aw --help
```

## Usage

```bash
# Analyze a website (human-readable output)
aw https://britannica.com

# JSON output
aw britannica.com --json

# Skip DNS and TLS probes
aw example.com --json --no-dns --no-tls

# Verbose logging
aw example.com -v
```

### Example Output

```json
{
  "domain": "britannica.com",
  "Content Management System": [
    { "technology": "WordPress", "version": "6.2" }
  ],
  "JavaScript Library": [
    { "technology": "jQuery", "version": "3.5.0" },
    { "technology": "React" }
  ],
  "Web Server": [
    { "technology": "Nginx", "version": "1.24.0" }
  ],
  "SSL/TLS certificate authority": [
    { "technology": "Let's Encrypt" }
  ]
}
```

## Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - How it works, detection methods, data flow
- **[Development](docs/DEVELOPMENT.md)** - Adding/editing technologies, testing, contributing
- **[Testing](docs/TESTING.md)** - Test coverage, running tests
- **[Releasing](docs/RELEASING.md)** - Build and release process
- **[Roadmap](docs/ROADMAP.md)** - Planned features

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

This includes both:
- The analyzer code
- Technology fingerprints from [Wappalyzer](https://github.com/tomnomnom/wappalyzer) (also GPL-3.0)

## Credits

- Fingerprint data: [Wappalyzer](https://github.com/tomnomnom/wappalyzer)
- Inspired by: [W3Techs](https://w3techs.com/)
