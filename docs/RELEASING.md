# Releasing Guide

This project uses [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated multi-platform releases.

## Quick Release Process

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

### 3. Automation Takes Over

The GitHub Actions workflow automatically:
- Builds binaries for all platforms (Linux, macOS Intel/ARM, Windows)
- Creates a GitHub Release with artifacts
- Generates shell installer scripts
- Publishes Homebrew formula to `gedex/homebrew-tap`

Monitor the workflow at: https://github.com/gedex/site-analyzer/actions

## Supported Platforms

- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `x86_64-pc-windows-msvc` (Windows)

## Installation Methods

Users can install via:

**Shell installer (Linux/macOS):**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gedex/site-analyzer/releases/latest/download/aw-installer.sh | sh
```

**Homebrew:**
```bash
brew install gedex/tap/aw
```

**Direct download:**
https://github.com/gedex/site-analyzer/releases

## Prerequisites

### Required GitHub Secret

Add this secret in repository settings (Settings → Secrets and variables → Actions):

- **`HOMEBREW_TAP_TOKEN`** - Personal access token with write access to `gedex/homebrew-tap`

Without this secret, the Homebrew publishing step will fail (but releases will still be created).

## Release Checklist

Before each release:

### Pre-Release

- [ ] All tests pass: `cargo test --workspace`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] No Clippy warnings: `cargo clippy --all-targets`
- [ ] Documentation updated (README.md, CHANGELOG.md if exists)
- [ ] Version bumped in `Cargo.toml` workspace.package section
- [ ] Test local build: `cargo dist build` (optional)

### Release

- [ ] Commit version bump: `git commit -am "Bump version to vX.Y.Z"`
- [ ] Create git tag: `git tag vX.Y.Z`
- [ ] Push commits: `git push origin trunk`
- [ ] Push tag: `git push origin vX.Y.Z`
- [ ] Monitor GitHub Actions: `.github/workflows/release.yml`

### Post-Release

- [ ] GitHub Release created automatically
- [ ] All platform binaries present in release
- [ ] Shell installer works (test on one platform)
- [ ] Homebrew formula published (if token configured)

## Testing Locally

Test the release process before pushing a tag:

```bash
# Preview what will be released
cargo dist plan

# Build all artifacts locally
cargo dist build

# Check generated manifest
cargo dist manifest
```

## cargo-dist Configuration

Configuration in `dist-workspace.toml`:

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

### Updating Configuration

If you modify `dist-workspace.toml`:

```bash
# Regenerate CI workflows
cargo dist generate

# Commit the changes
git add .github/workflows/release.yml dist-workspace.toml
git commit -m "Update cargo-dist configuration"
```

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR** - Breaking API changes
- **MINOR** - New features, backward compatible
- **PATCH** - Bug fixes, backward compatible

**Examples:**
- `0.1.0` → `0.1.1` - Bug fix
- `0.1.1` → `0.2.0` - Added new detection patterns
- `0.2.0` → `1.0.0` - First stable release
- `1.0.0` → `2.0.0` - Changed CLI arguments (breaking)

## Troubleshooting

### Release workflow fails

Check GitHub Actions logs. Common issues:

- **Missing `HOMEBREW_TAP_TOKEN`** - Add secret in repository settings
- **Invalid tag format** - Must be `vX.Y.Z` or `X.Y.Z`
- **Build failure on platform** - Check platform-specific build logs
- **Permission denied on tap** - Verify token has write access to homebrew-tap

### Test before releasing

```bash
# Dry-run the entire process
cargo dist plan

# Build locally to catch issues early
cargo dist build
```

### Regenerate workflows after cargo-dist update

```bash
# Update cargo-dist
cargo install cargo-dist --locked

# Regenerate workflows
cargo dist generate

# Commit changes
git add .github/ dist-workspace.toml
git commit -m "Update cargo-dist workflows"
```

## Resources

- **[cargo-dist Book](https://opensource.axo.dev/cargo-dist/book/)** - Full documentation
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
