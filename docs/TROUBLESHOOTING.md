# Troubleshooting

## `cargo` can't reach crates.io (timeout on `index.crates.io`)

Symptom:

```
warning: spurious network error … Timeout was reached (Failed to connect to index.crates.io port 443)
error: failed to get `chrono` as a dependency …
Caused by: download of config.json failed
```

This is a **network issue, not a code issue** — the Rust toolchain can't
download dependencies. The frontend (`VITE ready`) already works.

### 1. Retry first (often transient)

```powershell
cargo fetch --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

### 2. Check connectivity

```powershell
Test-NetConnection index.crates.io -Port 443
curl https://index.crates.io/config.json
```

- If both fail: check your internet, VPN/proxy, and firewall/antivirus
  (allow `cargo.exe` and `rustc.exe` through Windows Firewall).
- Try a different network (e.g. phone hotspot) to rule out ISP filtering.

### 3. Use a crates.io mirror (if crates.io is blocked/slow on your network)

Create `%USERPROFILE%\.cargo\config.toml`:

```toml
[source.crates-io]
replace-with = "rsproxy-sparse"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
```

Alternative mirror (USTC):

```toml
[source.crates-io]
replace-with = "ustc-sparse"

[source.ustc-sparse]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
```

Then:

```powershell
cargo fetch --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

> To undo, delete the `[source.*]` entries from `config.toml`.

### 4. Verify the toolchain

```powershell
rustc --version
cargo --version
rustup update stable
```
