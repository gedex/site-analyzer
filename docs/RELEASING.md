# Releasing Guide

This project uses [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated multi-platform releases.

## Quick Release Process (Automated)

### 1. Update Version

Edit the workspace version in `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Update this
```

### 2. Commit and Tag

```bash
git add Cargo.toml
git commit -m "Bump version to 0.2.0"
git tag v0.2.0
git push origin trunk
git push origin v0.2.0
```

### 3. Wait for Automation

The GitHub Actions workflow will automatically:
- Build binaries for all platforms (Linux, macOS Intel/ARM, Windows)
- Create a GitHub Release with all artifacts
- Generate shell installer scripts
- Publish Homebrew formula to `gedex/homebrew-tap`

### Supported Platforms

cargo-dist builds for:
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `x86_64-pc-windows-msvc` (Windows)

### Installation Methods (Automated)

**Shell installer (Linux/macOS):**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gedex/site-analyzer/releases/latest/download/aw-installer.sh | sh
```

**Homebrew:**
```bash
brew install gedex/tap/aw
```

**Direct download from GitHub Releases:**
Visit https://github.com/gedex/site-analyzer/releases

### Required GitHub Secrets

For Homebrew publishing to work, add this secret in repository settings:
- `HOMEBREW_TAP_TOKEN` - Personal access token with write access to `gedex/homebrew-tap`

## Building Release Binaries

### Development Build

```bash
cargo build
./target/debug/aw --help
```

Fast compilation, includes debug symbols, not optimized.

### Release Build

```bash
cargo build --release
./target/release/aw --help
```

**Characteristics:**
- Optimized binary (~5MB)
- 10-20x faster than debug
- No debug symbols
- Longer compilation time

### Build Artifacts

```
target/
├── debug/
│   └── aw                  # Debug binary (~30MB)
└── release/
    └── aw                  # Release binary (~5MB)
```

## Installation Methods

### 1. From Source (Recommended for Development)

```bash
git clone https://github.com/yourusername/site-analyzer.git
cd site-analyzer
cargo build --release
sudo cp target/release/aw /usr/local/bin/
```

### 2. Using cargo install

```bash
# Install from local path
cargo install --path crates/cli

# Or from crates.io (if published)
cargo install site-analyzer
```

**Note:** `cargo install` places binaries in `~/.cargo/bin/`, which should be in your PATH.

### 3. Prebuilt Binaries (via cargo-dist)

**Recommended: Shell installer (Linux/macOS)**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gedex/site-analyzer/releases/latest/download/aw-installer.sh | sh
```

**Homebrew (macOS/Linux)**

```bash
brew install gedex/tap/aw
```

**Manual download from GitHub Releases:**

Download the appropriate tarball from https://github.com/gedex/site-analyzer/releases/latest

```bash
# Linux x86_64
curl -L https://github.com/gedex/site-analyzer/releases/latest/download/aw-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv aw /usr/local/bin/

# macOS Intel
curl -L https://github.com/gedex/site-analyzer/releases/latest/download/aw-x86_64-apple-darwin.tar.gz | tar xz
sudo mv aw /usr/local/bin/

# macOS Apple Silicon
curl -L https://github.com/gedex/site-analyzer/releases/latest/download/aw-aarch64-apple-darwin.tar.gz | tar xz
sudo mv aw /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/gedex/site-analyzer/releases/latest/download/aw-x86_64-pc-windows-msvc.zip" -OutFile "aw.zip"
Expand-Archive aw.zip
Move-Item aw\aw.exe C:\Windows\System32\
```

## Cross-Platform Compilation

### Linux (x86_64)

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

### macOS (Intel)

```bash
cargo build --release --target x86_64-apple-darwin
```

### macOS (Apple Silicon)

```bash
cargo build --release --target aarch64-apple-darwin
```

### Windows

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

### Cross-Compilation Setup

Install cross-compilation tools:

```bash
# Install cross
cargo install cross

# Build for Linux from macOS
cross build --release --target x86_64-unknown-linux-gnu

# Build for Windows from macOS
cross build --release --target x86_64-pc-windows-gnu
```

## Version Management

### Update Version

Edit `Cargo.toml` files:

```toml
# crates/analyzer/Cargo.toml
[package]
name = "site-analyzer"
version = "0.2.0"  # Update this

# crates/cli/Cargo.toml
[package]
name = "aw"
version = "0.2.0"  # Update this

[dependencies]
site-analyzer = { path = "../analyzer", version = "0.2.0" }  # Update this
```

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR** - Breaking API changes
- **MINOR** - New features, backward compatible
- **PATCH** - Bug fixes, backward compatible

**Examples:**
- `0.1.0` → `0.1.1` - Bug fix
- `0.1.1` → `0.2.0` - Added new detection patterns
- `0.2.0` → `1.0.0` - First stable release
- `1.0.0` → `2.0.0` - Changed CLI arguments (breaking change)

## Publishing to crates.io

### Prerequisites

```bash
# Login to crates.io
cargo login

# Add metadata to Cargo.toml
```

### Required Metadata

```toml
[package]
name = "site-analyzer"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"
description = "Fast website technology detection tool"
homepage = "https://github.com/yourusername/site-analyzer"
repository = "https://github.com/yourusername/site-analyzer"
readme = "../README.md"
keywords = ["web", "analysis", "detection", "fingerprint"]
categories = ["web-programming", "command-line-utilities"]
```

### Publish

```bash
# Dry run (check for errors)
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

**Note:** Once published, versions are permanent and cannot be deleted.

## Creating GitHub Releases

### Automated (via cargo-dist)

Simply push a version tag and cargo-dist handles everything:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The `.github/workflows/release.yml` workflow will:
1. Build binaries for all platforms
2. Create GitHub Release with generated changelog
3. Upload all artifacts (tarballs, installers, checksums)
4. Publish to Homebrew tap

### Manual (for testing)

You can test the release process locally:

```bash
# Install cargo-dist if not already installed
cargo install cargo-dist

# Test the build process
cargo dist build

# Preview what will be released
cargo dist plan

# Generate installer scripts locally
cargo dist generate
```

## cargo-dist Configuration

The release process is configured in `dist-workspace.toml`:

```toml
[dist]
cargo-dist-version = "0.31.0"
ci = "github"
installers = ["shell", "homebrew"]
tap = "gedex/homebrew-tap"
targets = [
  "aarch64-apple-darwin",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc"
]
install-path = "CARGO_HOME"
publish-jobs = ["homebrew"]
```

To modify the release configuration:

```bash
# Update configuration
cargo dist init

# Regenerate CI workflows
cargo dist generate
```

## Release Checklist

Use this checklist before each release:

### Pre-Release

- [ ] All tests pass: `cargo test --workspace`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] No Clippy warnings: `cargo clippy --all-targets`
- [ ] Documentation updated (README.md, CHANGELOG.md)
- [ ] Version bumped in `Cargo.toml` workspace.package section
- [ ] CHANGELOG.md updated with changes
- [ ] Test local build: `cargo dist build`

### Publish (Automated via cargo-dist)

- [ ] Commit version bump: `git commit -am "Bump version to vX.Y.Z"`
- [ ] Create git tag: `git tag vX.Y.Z`
- [ ] Push commits: `git push origin trunk`
- [ ] Push tag: `git push origin vX.Y.Z`
- [ ] Monitor GitHub Actions workflow at `.github/workflows/release.yml`
- [ ] Verify `HOMEBREW_TAP_TOKEN` secret is configured

### Post-Release (Automated checks)

- [ ] GitHub Release created automatically with all artifacts
- [ ] Binaries available for all platforms (Linux, macOS, Windows)
- [ ] Shell installer script works: `curl ... | sh`
- [ ] Homebrew formula published to `gedex/homebrew-tap`
- [ ] Homebrew install works: `brew install gedex/tap/aw`
- [ ] Optional: Publish to crates.io: `cargo publish -p site-analyzer && cargo publish -p aw`

### Manual Verification

- [ ] Test installation on at least one platform
- [ ] Verify binary version: `aw --version`
- [ ] Announcement posted (if applicable)
- [ ] Documentation links work

## Distribution Options

### 1. GitHub Releases (Recommended)

**Pros:**
- Free hosting
- Built-in version management
- Easy to automate with Actions

**Cons:**
- Users must download manually

### 2. crates.io

**Pros:**
- Standard Rust distribution
- Users can `cargo install site-analyzer`
- Automatic dependency management

**Cons:**
- Requires compilation on user's machine
- Larger download (includes sources)

### 3. Homebrew (macOS/Linux)

**Automated via cargo-dist:**

Homebrew formulas are automatically generated and published to `gedex/homebrew-tap` when you push a version tag. The workflow handles:
- Formula generation with correct checksums
- Pushing to the tap repository
- Formula validation with `brew style`

Users can install with:
```bash
brew install gedex/tap/aw
```

**Note:** The `HOMEBREW_TAP_TOKEN` secret must be configured in repository settings for this to work.

### 4. Debian/Ubuntu Package

Create .deb package:

```bash
cargo install cargo-deb
cargo deb --target x86_64-unknown-linux-gnu
```

Generates: `target/debian/aw_0.1.0_amd64.deb`

Install:
```bash
sudo dpkg -i aw_0.1.0_amd64.deb
```

### 5. Docker Image

```dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/aw /usr/local/bin/aw
ENTRYPOINT ["aw"]
```

Build and publish:
```bash
docker build -t yourusername/site-analyzer:0.1.0 .
docker push yourusername/site-analyzer:0.1.0
```

Run:
```bash
docker run yourusername/site-analyzer:0.1.0 example.com --json
```

## Optimizing Binary Size

### Strip Debug Symbols

```toml
# Cargo.toml
[profile.release]
strip = true
lto = true
codegen-units = 1
opt-level = "z"  # Optimize for size
```

### Alternative: UPX Compression

```bash
# Install UPX
brew install upx  # macOS
sudo apt install upx  # Linux

# Compress binary
upx --best --lzma target/release/aw

# Result: ~5MB → ~2MB
```

**Note:** UPX may trigger some antivirus software.

## Troubleshooting

### Build Fails with Missing Dependencies

**Issue:** Linking errors on Linux

**Fix:** Install build dependencies
```bash
sudo apt install build-essential pkg-config libssl-dev
```

### Cross-Compilation Fails

**Issue:** Missing target

**Fix:** Install target
```bash
rustup target add x86_64-unknown-linux-gnu
```

### Binary Too Large

**Issue:** Debug build is 30MB

**Fix:** Use release build with optimizations
```bash
cargo build --release
strip target/release/aw
```

### macOS "Unidentified Developer" Warning

**Issue:** macOS Gatekeeper blocks unsigned binary

**Fix:** Sign the binary or instruct users to:
```bash
xattr -d com.apple.quarantine aw
```

## Resources

- **[cargo-dist](https://opensource.axo.dev/cargo-dist/)** - Primary distribution tool (automated releases)
- [cargo-dist Book](https://opensource.axo.dev/cargo-dist/book/) - Full documentation
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [cross](https://github.com/cross-rs/cross) - Cross-compilation tool (manual builds)
- [Semantic Versioning](https://semver.org/)

## Troubleshooting cargo-dist

### Release workflow fails

Check the GitHub Actions logs at `.github/workflows/release.yml`. Common issues:

- Missing `HOMEBREW_TAP_TOKEN` secret
- Invalid version tag format (must be `vX.Y.Z`)
- Build failures on specific platforms

### Test locally before pushing tag

```bash
# Dry-run the entire release process
cargo dist plan

# Build all artifacts locally
cargo dist build

# Check what would be uploaded
cargo dist manifest
```

### Regenerate workflows

If you update `dist-workspace.toml`:

```bash
cargo dist generate
git add .github/workflows/release.yml
git commit -m "Update cargo-dist workflows"
```
