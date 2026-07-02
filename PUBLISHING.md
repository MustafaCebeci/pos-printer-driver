# Publishing Guide

This document describes how to publish `pos-printer-driver` to npm and create a GitHub Release **manually** (no CI/CD).

> **Important**: This project does not use GitHub Actions or automated pipelines. You must build native binaries on each target platform yourself.

---

## Pre-release Checklist

Before publishing, verify everything is in order:

```bash
# 1. Ensure all tests pass
cargo test --workspace

# 2. Ensure no clippy warnings
cargo clippy --workspace

# 3. Ensure code is formatted
cargo fmt --check

# 4. Verify version numbers match across:
#    - Cargo.toml (workspace.package.version)
#    - crates/napi-binding/package.json (version)
#    - CHANGELOG.md (unreleased section header)
```

---

## Native Binary Compilation Strategy

Since there is no CI/CD, **you must compile the `.node` native addon on each target platform yourself**.

### Option A — Single Platform (recommended for v0.1.0)

Publish only for the platform you are currently on. Others can be added in later releases.

**Windows (x64):**
```bash
cd crates/napi-binding
npm run build
# Output: pos_printer_driver.node (in target/release or alongside package.json)
```

### Option B — Multi-platform (advanced)

To build for multiple targets without CI, you need the Rust target toolchains installed:

```bash
# Add targets
rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Build each (example for Linux from Windows with cross-compile):
# Note: cross-compiling USB drivers from Windows to Linux is complex and not guaranteed to work.
# Consider using a Linux VM or WSL.
```

For a proper multi-platform release, the recommended approach is:
1. Build Windows `.node` on Windows machine
2. Build Linux `.node` on a Linux machine or VM
3. Build macOS `.node` on a Mac (or borrow one)

---

## npm Publishing Steps

### 1. Login to npm (first time only)

```bash
npm login
```

### 2. Dry-run to verify package contents

```bash
cd crates/napi-binding
npm pack --dry-run
```

Verify that only `index.js`, `index.d.ts`, the `.node` binary for your platform, `README.md`, and `LICENSE` are included. The following must **NOT** be in the package:
- `crates/`
- `tests/`
- `target/`
- `Cargo.toml`
- `node_modules/` (dev dependencies should be stripped)

### 3. Update package.json for publishing

Before publishing, ensure `crates/napi-binding/package.json` has:

```json
{
  "name": "pos-printer-driver",
  "version": "0.1.0",
  "description": "Cross-platform ESC/POS printer driver for Node.js — TCP, Serial, and USB support",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "pos_printer_driver",
    "triples": {
      "defaults": true
    }
  },
  "files": ["index.js", "index.d.ts", "*.node"],
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/<YOUR_GITHUB_USERNAME>/pos-printer-driver.git"
  },
  "keywords": ["esc-pos", "pos-printer", "thermal-printer", "escpos", "napi-rs"],
  "engines": { "node": ">=10" }
}
```

> **Note**: The `repository` URL must point to your actual GitHub repository. Update `package.json` with your GitHub username and repo URL before publishing.

### 4. Publish to npm

```bash
cd crates/napi-binding
npm publish --access public
```

---

## GitHub Release Steps

### 1. Tag the release

```bash
git add .
git commit -m "Release v0.1.0"
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

### 2. Create the release on GitHub

1. Go to `https://github.com/<YOUR_GITHUB_USERNAME>/pos-printer-driver/releases`
2. Click **"Draft a new release"**
3. Select the `v0.1.0` tag
4. Set the release title: `v0.1.0`
5. Copy the relevant section from `CHANGELOG.md` into the description
6. (Optional) Attach `.node` binary files as release assets — this lets users download binaries directly

### 3. Attach binaries (optional but recommended)

If you built on multiple platforms, attach each platform's `.node` file:

- `pos_printer_driver-x86_64-pc-windows-msvc.node`
- `pos_printer_driver-aarch64-pc-windows-msvc.node`
- `pos_printer_driver-x86_64-unknown-linux-gnu.node`
- `pos_printer_driver-aarch64-unknown-linux-gnu.node`
- `pos_printer_driver-x86_64-apple-darwin.node`
- `pos_printer_driver-aarch64-apple-darwin.node`

---

## Post-release Checklist

After publishing:

- [ ] Verify the npm package is public: `npm view pos-printer-driver`
- [ ] Verify GitHub release is visible with correct tag
- [ ] (For multi-platform) Document in the release notes which platforms have binaries attached

---

## Version Bump for Next Release

When preparing v0.2.0:

1. Update version in `Cargo.toml` (workspace root):
   ```toml
   [workspace.package]
   version = "0.2.0"
   ```

2. Update version in `crates/napi-binding/package.json`:
   ```json
   { "version": "0.2.0" }
   ```

3. Add new section in `CHANGELOG.md`:
   ```md
   ## [0.2.0] - <date>

   ### Added
   ### Changed
   ### Fixed
   ```

4. Commit and tag: `git commit -m "Bump version to v0.2.0" && git tag v0.2.0`
