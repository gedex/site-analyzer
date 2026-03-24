# Releasing Guide

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

### 3. Prebuilt Binaries

Download from GitHub Releases:

```bash
# Linux
curl -L https://github.com/yourusername/site-analyzer/releases/download/v0.1.0/aw-linux-x86_64 -o aw
chmod +x aw
sudo mv aw /usr/local/bin/

# macOS
curl -L https://github.com/yourusername/site-analyzer/releases/download/v0.1.0/aw-macos-x86_64 -o aw
chmod +x aw
sudo mv aw /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/yourusername/site-analyzer/releases/download/v0.1.0/aw-macos-aarch64 -o aw
chmod +x aw
sudo mv aw /usr/local/bin/
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

### 1. Tag the Release

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

### 2. Build Binaries for All Platforms

```bash
# Linux
cross build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/aw aw-linux-x86_64

# macOS Intel
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/aw aw-macos-x86_64

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/aw aw-macos-aarch64

# Windows
cross build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/aw.exe aw-windows-x86_64.exe
```

### 3. Create Release on GitHub

1. Go to **Releases** → **Draft a new release**
2. Choose tag: `v0.1.0`
3. Title: `v0.1.0 - Initial Release`
4. Description:
   ```markdown
   ## Features
   - Detect 1,270+ web technologies
   - Fast async analysis
   - JSON and human-readable output

   ## Installation
   Download the binary for your platform:
   - **Linux:** `aw-linux-x86_64`
   - **macOS (Intel):** `aw-macos-x86_64`
   - **macOS (Apple Silicon):** `aw-macos-aarch64`
   - **Windows:** `aw-windows-x86_64.exe`
   ```
5. Upload binaries
6. Publish release

## Automated Releases with GitHub Actions

### .github/workflows/release.yml

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build for ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            binary: aw
            name: aw-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            binary: aw
            name: aw-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            binary: aw
            name: aw-macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-gnu
            binary: aw.exe
            name: aw-windows-x86_64.exe

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Rename binary
        run: |
          cp target/${{ matrix.target }}/release/${{ matrix.binary }} ${{ matrix.name }}

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Download artifacts
        uses: actions/download-artifact@v3

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            aw-linux-x86_64/aw-linux-x86_64
            aw-macos-x86_64/aw-macos-x86_64
            aw-macos-aarch64/aw-macos-aarch64
            aw-windows-x86_64.exe/aw-windows-x86_64.exe
          body: |
            ## Installation
            Download the binary for your platform and run:
            ```bash
            chmod +x aw-*
            sudo mv aw-* /usr/local/bin/aw
            ```
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Release Checklist

Use this checklist before each release:

### Pre-Release

- [ ] All tests pass: `cargo test --package site-analyzer`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] No Clippy warnings: `cargo clippy --all-targets`
- [ ] Documentation updated (README.md, CHANGELOG.md)
- [ ] Version bumped in both Cargo.toml files
- [ ] CHANGELOG.md updated with changes

### Build

- [ ] Release build succeeds: `cargo build --release`
- [ ] Binary size reasonable (~5MB)
- [ ] Test binary on target platforms
- [ ] Cross-compilation succeeds for all platforms

### Publish

- [ ] Git tag created: `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] Tag pushed: `git push origin v0.1.0`
- [ ] GitHub release created with binaries
- [ ] Published to crates.io (optional): `cargo publish`
- [ ] Release notes written with features and fixes

### Post-Release

- [ ] Installation instructions tested
- [ ] Binary downloads work from GitHub Releases
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

Create a tap:

```ruby
# Formula/aw.rb
class Aw < Formula
  desc "Fast website technology detection tool"
  homepage "https://github.com/yourusername/site-analyzer"
  url "https://github.com/yourusername/site-analyzer/archive/v0.1.0.tar.gz"
  sha256 "..."

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/aw"
  end
end
```

Install:
```bash
brew tap yourusername/tap
brew install aw
```

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

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [cargo-dist](https://github.com/axodotdev/cargo-dist) - Distribution tool
- [cross](https://github.com/cross-rs/cross) - Cross-compilation tool
- [GitHub Actions - Rust](https://github.com/actions-rs)
- [Semantic Versioning](https://semver.org/)
