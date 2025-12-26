# Auto Video Organize

### Overview
This repository focuses on consistent releases, reliable automation, and clear project documentation. The README stays intentionally high-level and avoids implementation details.

### Install
One-line install (builds from source):
```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Auto-Video-Organize/main/install.sh | bash
```

Requirements:
- `git`
- Rust toolchain (`cargo`)

Release binaries (recommended):
1. Download the asset matching your OS/arch from GitHub Releases.
2. Extract it (`.tar.gz` on macOS/Linux, `.zip` on Windows).
3. Move the binary to a directory on your `PATH` (e.g. `~/.local/bin`).

Source build (manual):
1. `git clone https://github.com/DennySORA/Auto-Video-Organize.git`
2. `cd Auto-Video-Organize`
3. `cargo build --release --locked`
4. Copy `target/release/auto_video_organize` to a directory on your `PATH`.

Script options:
- `REF` to select a tag/branch (default: `main`).
- `PREFIX` or `BIN_DIR` to change the install location.

### CI/CD
- Security scanning for dependency advisories
- Unit tests on every push and pull request
- Automatic Release creation when a tag is pushed (e.g. v1.2.3)
- Release assets include prebuilt binaries

### Contributing
Open an issue or pull request with a clear scope and rationale.

### Support
Use GitHub Issues for questions and requests.
