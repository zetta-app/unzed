#![allow(dead_code, unused_imports)]
// Telemetry log viewer removed for privacy.
// Stub module preserved so call sites compile.

use gpui::{App, Empty, EventEmitter, FocusHandle, Focusable, SharedString, Window, prelude::*};
use workspace::{
    Item, ItemHandle, ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView,
};

pub fn init(_cx: &mut App) {}

pub struct TelemetryLogView {
    focus_handle: FocusHandle,
}

pub enum TelemetryLogEvent {}

impl EventEmitter<TelemetryLogEvent> for TelemetryLogView {}

impl Item for TelemetryLogView {
    type Event = TelemetryLogEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Telemetry Log".into()
    }
}

impl Focusable for TelemetryLogView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TelemetryLogView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct TelemetryLogToolbarItemView;

impl TelemetryLogToolbarItemView {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for TelemetryLogToolbarItemView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl EventEmitter<ToolbarItemEvent> for TelemetryLogToolbarItemView {}

impl ToolbarItemView for TelemetryLogToolbarItemView {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        ToolbarItemLocation::Hidden
    }
}
