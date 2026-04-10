# UnZed

A privacy-oriented fork of [Zed](https://github.com/zed-industries/zed), the high-performance code editor. This fork removes all telemetry, crash reporting, cloud services, and phone-home behavior. No data leaves your machine unless you explicitly configure a third-party service.

---

## What changed

### Removed
- **Telemetry** — all event tracking, usage metrics, and diagnostics reporting gutted. The `telemetry::event!()` macro is a no-op.
- **Crash reporting** — Sentry minidump uploads removed. Local crash dumps still written for your own debugging.
- **Auto-update** — version check polling against zed.dev disabled. Default set to off.
- **Collaboration** — sign-in, WebSocket connections to zed.dev, and cloud connections disabled. Sign In button and collab panel hidden.
- **Zed AI edit predictions** — cloud-based Zed/Mercury prediction providers disabled. Upsell modal suppressed.
- **Settings UI sections** — Privacy (telemetry toggles) and Collaboration pages removed from the settings GUI.
- **Tracking headers** — `x-zed-system-id` and `x-zed-metrics-id` stripped from all requests.
- **Tracking query params** — `metrics_id`, `system_id`, `is_staff` removed from update and LLM token requests.

### Kept
- **Ollama** edit predictions (fully local, `localhost:11434`)
- **OpenAI-compatible API** edit predictions (connects to your configured URL)
- **Copilot** and **Codestral** edit predictions (connect to GitHub/Mistral, not zed.dev)
- **All language model providers** (Ollama, OpenAI, Anthropic, etc. — user-configured)
- **Extension downloads** from the Zed extension registry
- **All editor features** — LSP, terminal, git, debugging, search, etc.
- **Local hang monitoring** — writes traces to disk for debugging, never uploads

### Default settings changed
| Setting | Upstream | This fork |
|---------|----------|-----------|
| `telemetry.diagnostics` | `true` | `false` |
| `telemetry.metrics` | `true` | `false` |
| `auto_update` | `true` | `false` |
| `edit_predictions.provider` | `"zed"` | `"none"` |
| `title_bar.show_sign_in` | `true` | `false` |
| `collaboration_panel.button` | `true` | `false` |

## Building

### System dependencies (Linux)

```
sudo apt install libglib2.0-dev libgtk-3-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev libx11-dev libx11-xcb-dev libxcb1-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libfontconfig-dev \
  libvulkan-dev libssl-dev
```

### Build

```
cargo build -p zed --release
```

The binary will be at `target/release/zed`.

## Detailed changelog

See [PRIVACY_CHANGES.md](./PRIVACY_CHANGES.md) for the full list of modified files and technical details.

## Upstream

Based on [Zed](https://github.com/zed-industries/zed) by Zed Industries, Inc. Licensed under GPL-3.0-or-later / Apache-2.0 / AGPL-3.0-or-later (see LICENSE files).
