# 🦀 Git Commit Rewriter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](https://github.com/atabits/git-commit-rewriter)

> **Edit Git commit messages the easy way** — A beautiful, fast GUI app built with Rust

![Screenshot](images/screenshot-v2.1.0.png)

---

## ✨ Why You'll Love It

- **🎨 Beautiful Interface** — Clean, modern UI that's actually enjoyable to use
- **⚡️ Blazingly Fast** — Native Rust performance, launches in under 500ms
- **💾 Super Lightweight** — Only ~3MB in size, uses minimal memory
- **🔍 Smart Search** — Find commits instantly by message, hash, or author
- **🛡️ Safe by Default** — Automatic backups before any changes
- **🔄 Easy Rollback** — Undo changes with a single click
- **🌐 Cross-Platform** — Works on macOS, Linux, and Windows

---

## 🚀 Quick Start

### Download

Prebuilt binaries are attached to every [release](https://github.com/atabits/git-commit-rewriter/releases/latest):

| Platform | File |
|---|---|
| macOS (Apple Silicon / Intel) | `commit-rewriter-<version>-macos-arm64.dmg` / `-macos-x86_64.dmg` |
| Windows | `commit-rewriter-<version>-windows-portable.zip` |
| Linux (x86_64 / ARM64) | `commit-rewriter-<version>-linux-<arch>.AppImage` or `.tar.gz` |

### Build from Source

```bash
git clone https://github.com/atabits/git-commit-rewriter.git
cd git-commit-rewriter
cargo build --release
```

The app will be ready in `target/release/commit-rewriter` ✨

### Install Globally

```bash
# macOS / Linux
sudo cp target/release/commit-rewriter /usr/local/bin/

# Now run from anywhere
commit-rewriter
```

---

## 📖 How to Use

1. **🚀 Launch** — Run `commit-rewriter` from your terminal
2. **📂 Open Repo** — Click "Select Repository" and choose your project
3. **🔍 Find Commit** — Browse or search for the commit to edit
4. **✏️ Edit Message** — Type your new commit message
5. **✅ Apply** — Click "Apply Change" and review the preview
6. **🎉 Done!** — Confirm changes or rollback if needed

> **Tip:** After editing, push with `git push --force-with-lease`

---

## ⚠️ Important Notes

**⚡️ History rewriting changes commit hashes**

- 💾 Always backup your repository first
- 👥 Coordinate with your team before rewriting shared branches
- 🔒 Use `git push --force-with-lease` to push safely

**🛡️ Built-in Safety**

- By default only the current branch is rewritten. Enable **All branches and tags** to also update every other branch and tag that contains the commit.
- The full commit message is edited, including its body and trailers such as `Signed-off-by:`. Authors, dates and file contents stay the same.
- Before moving any ref, the app saves its original value under `refs/commit-rewriter/backup/`. **Confirm** deletes the backups, **Cancel** restores the refs. If the app is closed before you decide, it asks again the next time the repository is opened.
- Remote-tracking branches are never touched, and commit signatures (GPG/SSH) are not preserved on rewritten commits.

---

## 🛠️ For Developers

Need **Rust 1.95+**. Git itself is not required: the app works through libgit2.

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Formatting, lints and tests (same checks as CI)
make check
```

### 📦 Build for Distribution

```bash
# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-gnu

# Linux (.tar.gz + .AppImage, run on Linux)
make linux
```

On Linux, building requires a C toolchain and pkg-config:

```bash
# Debian / Ubuntu
sudo apt install build-essential pkg-config

# Arch
sudo pacman -S base-devel pkgconf
```

App icons are generated from `assets/icon/icon.svg` with `./scripts/generate-icons.sh` (needs librsvg and ImageMagick); the results are committed, so regular builds don't need these tools.

The folder picker uses [xdg-desktop-portal](https://flatpak.github.io/xdg-desktop-portal/), which is available on most desktop environments.

---

## 📄 License

Released under the [MIT License](LICENSE) — free to use and modify!

---

## 💬 Get in Touch

**Built by Amin Atabiev**

- 📧 Email: [endlesspring.of@gmail.com](mailto:endlesspring.of@gmail.com)
- 🐙 GitHub: [@atabits](https://github.com/atabits)

---

<div align="center">

**⭐️ If you like this project, give it a star!**

Made with ❤️ and 🦀 Rust

</div>
