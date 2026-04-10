# Privacy-Oriented Fork Changes

This fork removes all telemetry, crash reporting, cloud services, and phone-home
behavior from Zed. No data leaves your machine unless you explicitly configure
a third-party service (e.g., Ollama, OpenAI-compatible API).

## What was removed

### Telemetry
- `telemetry::event!()` macro replaced with a no-op shim (zero dependencies)
- All 162+ event call sites across 30+ crates now compile but do nothing
- Event queuing, HTTP flushing, and SHA256 checksum verification removed from `client/src/telemetry.rs`
- Event coalescer deleted
- `metrics_enabled()` and `diagnostics_enabled()` always return `false`
- Default telemetry settings changed to `false`/`false`
- Telemetry log viewer replaced with empty stub
- `telemetry_events` dependency removed from `zed` crate
- `sha2` and `regex` dependencies removed from `client` crate

### Crash Reporting (Sentry)
- Minidump upload to Sentry completely removed from `reliability.rs`
- Build timing upload removed
- `MINIDUMP_ENDPOINT` constant removed
- `reqwest` dependency removed from `zed` crate (was only used for multipart crash uploads)
- Local hang monitoring preserved (writes traces to disk only, never uploads)
- Local crash dump files still written for debugging but never sent anywhere

### Auto-Update
- Auto-updater initialization disabled — no `AutoUpdater` instance created
- No version check polling against `cloud.zed.dev/releases/`
- `metrics_id`, `system_id`, `is_staff` fields stripped from update request query params
- Default `auto_update` setting changed to `false`

### Collaboration (Collab)
- `sign_in_with_optional_connect()` returns immediately without connecting
- `connect()` returns immediately without connecting
- No WebSocket connection to zed.dev RPC servers
- No cloud WebSocket connection
- No browser-based authentication flow triggered
- `x-zed-system-id` and `x-zed-metrics-id` headers removed from WebSocket handshake

### Zed AI / Cloud Edit Predictions
- `EditPredictionProvider::Zed` (Zeta) disabled — maps to `None` at runtime
- `EditPredictionProvider::Mercury` disabled — maps to `None` at runtime
- Zed AI removed from available providers list in UI
- Default edit prediction provider changed from `"zed"` to `"none"`
- Zed Predict upsell modal disabled (`should_show_upsell_modal` always returns `false`)
- Cloud experiments fetch disabled
- `system_id` stripped from all LLM token acquisition methods

### What still works
- Ollama edit predictions (fully local, connects to `localhost:11434`)
- OpenAI-compatible API edit predictions (connects to user-configured URL)
- Copilot edit predictions (connects to GitHub, not zed.dev)
- Codestral edit predictions (connects to Mistral, not zed.dev)
- All language model providers (Ollama, OpenAI, Anthropic, etc. — user-configured)
- Extension downloads from zed.dev extension registry
- All editor features, LSP, terminal, git, debugging, etc.
- Local crash dump files for debugging

## Files modified

| File | Change |
|------|--------|
| `crates/telemetry/src/telemetry.rs` | Replaced with no-op macro shim |
| `crates/telemetry/Cargo.toml` | Stripped all dependencies |
| `crates/client/src/telemetry.rs` | Gutted: all methods are no-ops |
| `crates/client/src/telemetry/event_coalescer.rs` | Deleted |
| `crates/client/src/client.rs` | Disabled collab, stripped tracking headers and system_id from LLM tokens |
| `crates/client/Cargo.toml` | Removed `telemetry`, `sha2`, `regex` deps |
| `crates/zed/src/reliability.rs` | Removed crash upload, kept local hang monitoring |
| `crates/zed/src/zed/telemetry_log.rs` | Replaced with empty stub |
| `crates/zed/Cargo.toml` | Removed `reqwest`, `telemetry_events` deps |
| `crates/auto_update/src/auto_update.rs` | Disabled updater init, stripped tracking from queries |
| `crates/zed/src/zed/edit_prediction_registry.rs` | Disabled Zed/Mercury cloud providers |
| `crates/edit_prediction_ui/src/edit_prediction_button.rs` | Removed Zed/Mercury from provider list |
| `crates/edit_prediction/src/edit_prediction.rs` | Disabled upsell modal and experiments fetch |
| `crates/settings_content/src/settings_content.rs` | Telemetry defaults to `false` |
| `assets/settings/default.json` | `telemetry` defaults `false`, `auto_update` `false`, provider `"none"` |

## Build requirements (Linux)

```
sudo apt install libglib2.0-dev libgtk-3-dev libxkbcommon-dev libwayland-dev \
  libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libasound2-dev libfontconfig-dev libvulkan-dev libssl-dev
```
