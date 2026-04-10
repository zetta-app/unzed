// Telemetry sending has been removed for privacy. The Telemetry struct and its
// public API are preserved as no-ops so that the rest of the codebase compiles.

use crate::TelemetrySettings;
use clock::SystemClock;
use fs::Fs;
use futures::channel::mpsc;
use gpui::{App, Task};
use http_client::HttpClientWithUrl;
use parking_lot::Mutex;
use settings::{Settings, SettingsStore};
use std::collections::HashSet;
use std::sync::Arc;
use std::path::PathBuf;
use telemetry_events::{AssistantEventData, EventWrapper};
use worktree::{UpdatedEntriesSet, WorktreeId};

pub struct TelemetrySubscription {
    pub historical_events: anyhow::Result<HistoricalEvents>,
    pub queued_events: Vec<EventWrapper>,
    pub live_events: mpsc::UnboundedReceiver<EventWrapper>,
}

pub struct HistoricalEvents {
    pub events: Vec<EventWrapper>,
    pub parse_error_count: usize,
}

pub struct Telemetry {
    state: Arc<Mutex<TelemetryState>>,
}

struct TelemetryState {
    settings: TelemetrySettings,
    system_id: Option<Arc<str>>,
    installation_id: Option<Arc<str>>,
    metrics_id: Option<Arc<str>>,
    is_staff: Option<bool>,
    _worktrees_with_project_type_events_sent: HashSet<WorktreeId>,
}

pub fn os_name() -> String {
    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        format!("Linux {}", gpui::guess_compositor())
    }
    #[cfg(target_os = "freebsd")]
    {
        format!("FreeBSD {}", gpui::guess_compositor())
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
}

pub fn os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::NSProcessInfo;
        let process_info = NSProcessInfo::processInfo();
        process_info
            .operatingSystemVersionString()
            .to_string()
            .replace("Version ", "")
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use std::path::Path;
        let content = if let Ok(file) = std::fs::read_to_string(Path::new("/etc/os-release")) {
            file
        } else if let Ok(file) = std::fs::read_to_string(Path::new("/usr/lib/os-release")) {
            file
        } else if let Ok(file) = std::fs::read_to_string(Path::new("/var/run/os-release")) {
            file
        } else {
            "".to_string()
        };
        let mut name = "unknown";
        let mut version = "unknown";
        for line in content.lines() {
            match line.split_once('=') {
                Some(("ID", val)) => name = val.trim_matches('"'),
                Some(("VERSION_ID", val)) => version = val.trim_matches('"'),
                _ => {}
            }
        }
        format!("{} {}", name, version)
    }
    #[cfg(target_os = "windows")]
    {
        let mut info = unsafe { std::mem::zeroed() };
        let status = unsafe { windows::Wdk::System::SystemServices::RtlGetVersion(&mut info) };
        if status.is_ok() {
            semver::Version::new(
                info.dwMajorVersion as _,
                info.dwMinorVersion as _,
                info.dwBuildNumber as _,
            )
            .to_string()
        } else {
            "unknown".to_string()
        }
    }
}

impl Telemetry {
    pub fn new(
        _clock: Arc<dyn SystemClock>,
        _client: Arc<HttpClientWithUrl>,
        cx: &mut App,
    ) -> Arc<Self> {
        let state = Arc::new(Mutex::new(TelemetryState {
            settings: *TelemetrySettings::get_global(cx),
            system_id: None,
            installation_id: None,
            metrics_id: None,
            is_staff: None,
            _worktrees_with_project_type_events_sent: HashSet::new(),
        }));

        cx.observe_global::<SettingsStore>({
            let state = state.clone();
            move |cx| {
                state.lock().settings = *TelemetrySettings::get_global(cx);
            }
        })
        .detach();

        Arc::new(Self { state })
    }

    pub fn log_file_path() -> PathBuf {
        paths::logs_dir().join("telemetry.log")
    }

    pub async fn subscribe_with_history(
        self: &Arc<Self>,
        _fs: Arc<dyn Fs>,
    ) -> TelemetrySubscription {
        let (_tx, rx) = mpsc::unbounded();
        TelemetrySubscription {
            historical_events: Ok(HistoricalEvents {
                events: Vec::new(),
                parse_error_count: 0,
            }),
            queued_events: Vec::new(),
            live_events: rx,
        }
    }

    pub fn has_checksum_seed(&self) -> bool {
        false
    }

    pub fn start(
        self: &Arc<Self>,
        system_id: Option<String>,
        installation_id: Option<String>,
        _session_id: String,
        _cx: &App,
    ) {
        let mut state = self.state.lock();
        state.system_id = system_id.map(|id| id.into());
        state.installation_id = installation_id.map(|id| id.into());
    }

    pub fn metrics_enabled(self: &Arc<Self>) -> bool {
        false
    }

    pub fn diagnostics_enabled(self: &Arc<Self>) -> bool {
        false
    }

    pub fn set_authenticated_user_info(
        self: &Arc<Self>,
        metrics_id: Option<String>,
        is_staff: bool,
    ) {
        let mut state = self.state.lock();
        state.metrics_id = metrics_id.map(|id| id.into());
        state.is_staff = Some(is_staff);
    }

    pub fn report_assistant_event(self: &Arc<Self>, _event: AssistantEventData) {}

    pub fn log_edit_event(self: &Arc<Self>, _environment: &'static str, _is_via_ssh: bool) {}

    pub fn report_discovered_project_type_events(
        self: &Arc<Self>,
        _worktree_id: WorktreeId,
        _updated_entries_set: &UpdatedEntriesSet,
    ) {
    }

    pub fn metrics_id(self: &Arc<Self>) -> Option<Arc<str>> {
        None
    }

    pub fn system_id(self: &Arc<Self>) -> Option<Arc<str>> {
        None
    }

    pub fn installation_id(self: &Arc<Self>) -> Option<Arc<str>> {
        self.state.lock().installation_id.clone()
    }

    pub fn is_staff(self: &Arc<Self>) -> Option<bool> {
        None
    }

    pub fn flush_events(self: &Arc<Self>) -> Task<()> {
        Task::ready(())
    }
}
