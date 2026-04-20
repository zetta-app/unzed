# Zed Telemetry & External Services Audit

This document catalogs all telemetry, crash reporting, and external service calls
found in the Zed codebase, prior to their removal for a privacy-oriented build.

## 1. Telemetry Pipeline

### Crates involved
- `crates/telemetry/` — `telemetry::event!()` macro and in-process event queue
- `crates/client/src/telemetry.rs` — `Telemetry` struct, batches and flushes events over HTTP
- `crates/telemetry_events/` — event data structures (`EventRequestBody`, `EventWrapper`, `Event`)

### How it works
- `telemetry::event!("Name", key = value)` pushes to an in-memory mpsc queue
- `Telemetry::report_event` wraps events with metadata and queues them
- `Telemetry::flush_events` sends `POST /telemetry/events` to `api.zed.dev`
- Flush triggers: every 5 min or 50 events (1s / 5 events in debug)
- Events also written to local `telemetry.log` file
- Requests include `x-zed-checksum` header (SHA256 HMAC with `ZED_CLIENT_CHECKSUM_SEED`)

### Data sent per request (`EventRequestBody`)
- `system_id` — per-machine
- `installation_id` — per-installation (differs for stable/preview/dev)
- `session_id` — per-launch
- `metrics_id` — per-signed-in user
- `is_staff`, `app_version`, `os_name`, `os_version`, `architecture`, `release_channel`

### User settings
```json
{ "telemetry": { "diagnostics": false, "metrics": false } }
```

## 2. Tracked Events (telemetry::event! call sites)

- App lifecycle: `App First Opened`, `App Opened`, `App Closed`, `App First Opened For Release Channel`
- Settings: `Settings Changed`, `Settings Viewed`, `Settings Closed`, `Settings Searched`, `Settings Change`, `Settings Error Shown`, `Settings Navigation Clicked`, `Setting Project Clicked`
- Projects: `Project Opened` (with project_type), `SSH Project Opened/Created`, `SSH Server Created`, `WSL Distro Added`
- Editor: `Editor Edited` (coalesced), `Panel Button Clicked`, `Project Panel Updated`
- AI/Assistant: `Assistant Invoked`, `Assistant Responded`, `Assistant Response Accepted/Rejected`
- Edit predictions: `Edit Prediction Provider Changed`, `Kernel Status Changed`
- Collaboration: `Screen Share Enabled/Disabled`, `Microphone Enabled/Disabled`
- Onboarding: `Onboarding Page Opened`, `Finish Setup`, `Banner Clicked/Dismissed`
- Reliability: `Minidump Uploaded`, `Build Timing: Cargo Build`
- Workspace trust: `Open in Restricted`, `Trust and Continue`, `Dismissed`

## 3. Zed Cloud Endpoints

Default `server_url`: `https://zed.dev` (in `assets/settings/default.json`, overridable via `ZED_SERVER_URL`)

| Base URL | Mapped API URL | Purpose |
|---|---|---|
| `https://zed.dev` | `https://api.zed.dev` | Telemetry, auth, general API |
| `https://zed.dev` | `https://cloud.zed.dev` | Auto-update, LLM proxy, extensions |
| `https://staging.zed.dev` | `https://api-staging.zed.dev` | Staging API |
| `https://staging.zed.dev` | `https://llm-staging.zed.dev` | Staging LLM |

### Specific endpoints
- `POST /telemetry/events` — telemetry
- `GET /rpc` → WebSocket redirect for collab
- `GET /native_app_signin` — browser auth flow
- `GET /releases/{channel}/{version}/asset` — auto-update (sends metrics_id, system_id, is_staff)
- LLM token acquisition/refresh endpoints
- Extension registry endpoints

## 4. Crash Reporting (Sentry)

- Minidumps uploaded to `ZED_MINIDUMP_ENDPOINT` (Sentry minidump URL)
- Sentry org: `zed-dev`, project: `zed`
- Metadata: panic message, channel, version, binary, metrics_id/installation_id, staff status
- Old endpoints (`/telemetry/crashes`, `/panics`, `/hangs`) now just return OK
- Controlled by `telemetry.diagnostics` setting
- Implementation: `crates/zed/src/reliability.rs`

## 5. Server-Side Forwarding (collab)

`crates/collab/src/api/events.rs`:
- Receives telemetry events from clients
- Forwards to AWS Kinesis → Snowflake data warehouse
- Amplitude reads from this pipeline (referenced in code comments)

## 6. Auto-Update

`crates/auto_update/src/auto_update.rs`:
- Polls `cloud.zed.dev/releases/{channel}/{version}/asset`
- Sends `metrics_id`, `system_id`, `is_staff` as query params when metrics enabled

## 7. Other External Services

- `https://openrouter.ai/api/v1` — OpenRouter LLM provider
- GitHub API — git hosting, extension downloads
- GitLab, Gitea, Forgejo, Bitbucket, Gitee APIs — git hosting integrations
- GitHub Copilot — edit prediction provider
- Codestral (Mistral) — edit prediction provider
- User-configured LLM APIs: OpenAI, Anthropic, Google, DeepSeek, Groq, xAI, etc.
- Ollama, LM Studio — local LLM providers (not external calls unless configured)

## 8. Docs Site Analytics

`crates/docs_preprocessor/`: injects `DOCS_AMPLITUDE_API_KEY` into docs website (website-only, not in editor)

---

## Files to modify for removal

### Core telemetry removal
- `crates/telemetry/src/telemetry.rs` — gut the macro to no-op
- `crates/client/src/telemetry.rs` — remove HTTP sending, flush logic, checksum
- `crates/telemetry_events/src/telemetry_events.rs` — keep structs if needed for compilation, remove sending
- `crates/zed/src/zed/telemetry_log.rs` — remove or stub the telemetry log viewer
- `crates/settings_content/src/settings_content.rs` — remove TelemetrySettingsContent
- `crates/client/src/client.rs` — remove TelemetrySettings

### Crash reporting removal
- `crates/zed/src/reliability.rs` — remove minidump upload, Sentry integration
- `crates/client/src/telemetry.rs` — remove MINIDUMP_ENDPOINT

### Zed cloud / LLM removal
- `crates/http_client/src/http_client.rs` — remove `build_zed_cloud_url`, `build_zed_llm_url`, `build_zed_api_url`
- `crates/client/src/client.rs` — remove `acquire_llm_token`, `refresh_llm_token`, cloud connection code
- `crates/auto_update/src/auto_update.rs` — remove metrics_id/system_id from update requests

### Server-side (collab)
- `crates/collab/src/api/events.rs` — remove Kinesis/Snowflake forwarding

### All `telemetry::event!()` call sites
- ~50+ files across workspace, title_bar, settings_ui, repl, onboarding, project_panel, theme_selector, recent_projects, etc.
