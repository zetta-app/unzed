use anyhow::Result;
use collections::{BTreeMap, HashMap};
use credentials_provider::CredentialsProvider;
use qwen::QWEN_API_URL;

use futures::Stream;
use futures::{FutureExt, StreamExt, future::BoxFuture, stream::BoxStream};
use gpui::{AnyView, App, AsyncApp, Context, Entity, SharedString, Task, Window};
use http_client::HttpClient;
use language_model::{
    ApiKeyState, AuthenticateError, EnvVar, IconOrSvg, LanguageModel, LanguageModelCompletionError,
    LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProvider,
    LanguageModelProviderId, LanguageModelProviderName, LanguageModelProviderState,
    LanguageModelRequest, LanguageModelToolChoice, LanguageModelToolResultContent,
    LanguageModelToolUse, MessageContent, RateLimiter, Role, StopReason, TokenUsage, env_var,
};
pub use settings::QwenAvailableModel as AvailableModel;
use settings::{Settings, SettingsStore};
use std::pin::Pin;
use std::sync::{Arc, LazyLock};

use ui::{ButtonLink, ConfiguredApiCard, List, ListBulletItem, prelude::*};
use ui_input::InputField;
use util::ResultExt;

use language_model::util::{fix_streamed_json, parse_tool_arguments};

const PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("qwen");
const PROVIDER_NAME: LanguageModelProviderName = LanguageModelProviderName::new("Qwen");

const API_KEY_ENV_VAR_NAME: &str = "DASHSCOPE_API_KEY";
static API_KEY_ENV_VAR: LazyLock<EnvVar> = env_var!(API_KEY_ENV_VAR_NAME);

#[derive(Default)]
struct RawToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct QwenSettings {
    pub api_url: String,
    pub available_models: Vec<AvailableModel>,
}

pub struct QwenLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    state: Entity<State>,
}

pub struct State {
    api_key_state: ApiKeyState,
    credentials_provider: Arc<dyn CredentialsProvider>,
}

impl State {
    fn is_authenticated(&self) -> bool {
        self.api_key_state.has_key()
    }

    fn set_api_key(&mut self, api_key: Option<String>, cx: &mut Context<Self>) -> Task<Result<()>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = QwenLanguageModelProvider::api_url(cx);
        self.api_key_state.store(
            api_url,
            api_key,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        )
    }

    fn authenticate(&mut self, cx: &mut Context<Self>) -> Task<Result<(), AuthenticateError>> {
        let credentials_provider = self.credentials_provider.clone();
        let api_url = QwenLanguageModelProvider::api_url(cx);
        self.api_key_state.load_if_needed(
            api_url,
            |this| &mut this.api_key_state,
            credentials_provider,
            cx,
        )
    }
}

impl QwenLanguageModelProvider {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        credentials_provider: Arc<dyn CredentialsProvider>,
        cx: &mut App,
    ) -> Self {
        let state = cx.new(|cx| {
            cx.observe_global::<SettingsStore>(|this: &mut State, cx| {
                let credentials_provider = this.credentials_provider.clone();
                let api_url = Self::api_url(cx);
                this.api_key_state.handle_url_change(
                    api_url,
                    |this| &mut this.api_key_state,
                    credentials_provider,
                    cx,
                );
                cx.notify();
            })
            .detach();
            State {
                api_key_state: ApiKeyState::new(Self::api_url(cx), (*API_KEY_ENV_VAR).clone()),
                credentials_provider,
            }
        });

        Self { http_client, state }
    }

    fn create_language_model(
        &self,
        model: qwen::Model,
        settings_model: Option<&AvailableModel>,
    ) -> Arc<dyn LanguageModel> {
        Arc::new(QwenLanguageModel {
            id: LanguageModelId::from(model.id().to_string()),
            model,
            settings_model: settings_model.cloned(),
            state: self.state.clone(),
            http_client: self.http_client.clone(),
            request_limiter: RateLimiter::new(4),
        })
    }

    fn settings(cx: &App) -> &QwenSettings {
        &crate::AllLanguageModelSettings::get_global(cx).qwen
    }

    fn api_url(cx: &App) -> SharedString {
        let api_url = &Self::settings(cx).api_url;
        if api_url.is_empty() {
            QWEN_API_URL.into()
        } else {
            SharedString::new(api_url.as_str())
        }
    }
}

impl LanguageModelProviderState for QwenLanguageModelProvider {
    type ObservableEntity = State;

    fn observable_entity(&self) -> Option<Entity<Self::ObservableEntity>> {
        Some(self.state.clone())
    }
}

impl LanguageModelProvider for QwenLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn icon(&self) -> IconOrSvg {
        IconOrSvg::Icon(IconName::AiQwen)
    }

    fn default_model(&self, _cx: &App) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(qwen::Model::default(), None))
    }

    fn default_fast_model(&self, _cx: &App) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(qwen::Model::default_fast(), None))
    }

    fn provided_models(&self, cx: &App) -> Vec<Arc<dyn LanguageModel>> {
        let settings = Self::settings(cx);
        let settings_by_name: HashMap<&str, &AvailableModel> = settings
            .available_models
            .iter()
            .map(|m| (m.name.as_str(), m))
            .collect();

        let mut models = BTreeMap::default();

        let builtins: Vec<(&str, qwen::Model)> = vec![
            ("qwen3-235b-a22b", qwen::Model::Qwen3_235B),
            ("qwen3-32b", qwen::Model::Qwen3_32B),
            ("qwen3-coder-plus", qwen::Model::Qwen3CoderPlus),
            ("qwen-plus", qwen::Model::QwenPlus),
        ];

        for (name, model) in builtins {
            let settings_model = settings_by_name.get(name).copied();
            models.insert(name, (model, settings_model));
        }

        for available_model in settings.available_models.iter() {
            if models.contains_key(available_model.name.as_str()) {
                continue;
            }
            models.insert(
                &available_model.name,
                (
                    qwen::Model::Custom {
                        name: available_model.name.clone(),
                        display_name: available_model.display_name.clone(),
                        max_tokens: available_model.max_tokens,
                        max_output_tokens: available_model.max_output_tokens,
                    },
                    Some(available_model),
                ),
            );
        }

        models
            .into_values()
            .map(|(model, settings)| self.create_language_model(model, settings))
            .collect()
    }

    fn is_authenticated(&self, cx: &App) -> bool {
        self.state.read(cx).is_authenticated()
    }

    fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
        self.state.update(cx, |state, cx| state.authenticate(cx))
    }

    fn configuration_view(
        &self,
        _target_agent: language_model::ConfigurationViewTargetAgent,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyView {
        cx.new(|cx| ConfigurationView::new(self.state.clone(), window, cx))
            .into()
    }

    fn reset_credentials(&self, cx: &mut App) -> Task<Result<()>> {
        self.state
            .update(cx, |state, cx| state.set_api_key(None, cx))
    }
}

pub struct QwenLanguageModel {
    id: LanguageModelId,
    model: qwen::Model,
    settings_model: Option<AvailableModel>,
    state: Entity<State>,
    http_client: Arc<dyn HttpClient>,
    request_limiter: RateLimiter,
}

impl QwenLanguageModel {
    fn stream_completion(
        &self,
        request: qwen::Request,
        cx: &AsyncApp,
    ) -> BoxFuture<'static, Result<BoxStream<'static, Result<qwen::StreamResponse>>>> {
        let http_client = self.http_client.clone();

        let (api_key, api_url) = self.state.read_with(cx, |state, cx| {
            let api_url = QwenLanguageModelProvider::api_url(cx);
            (state.api_key_state.key(&api_url), api_url)
        });

        let future = self.request_limiter.stream(async move {
            let Some(api_key) = api_key else {
                return Err(LanguageModelCompletionError::NoApiKey {
                    provider: PROVIDER_NAME,
                });
            };
            let request =
                qwen::stream_completion(http_client.as_ref(), &api_url, &api_key, request);
            let response = request.await?;
            Ok(response)
        });

        async move { Ok(future.await?.boxed()) }.boxed()
    }
}

impl LanguageModel for QwenLanguageModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(self.model.display_name().to_string())
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn supports_tools(&self) -> bool {
        self.settings_model
            .as_ref()
            .map(|m| m.capabilities.tools)
            .unwrap_or(true)
    }

    fn supports_streaming_tools(&self) -> bool {
        true
    }

    fn supports_split_token_display(&self) -> bool {
        true
    }

    fn supports_tool_choice(&self, _choice: LanguageModelToolChoice) -> bool {
        self.supports_tools()
    }

    fn supports_images(&self) -> bool {
        self.settings_model
            .as_ref()
            .map(|m| m.capabilities.images)
            .unwrap_or(false)
    }

    fn supports_thinking(&self) -> bool {
        self.settings_model
            .as_ref()
            .and_then(|m| m.enable_thinking)
            .unwrap_or(false)
    }

    fn telemetry_id(&self) -> String {
        format!("qwen/{}", self.model.id())
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_token_count()
    }

    fn max_output_tokens(&self) -> Option<u64> {
        self.settings_model
            .as_ref()
            .and_then(|m| m.max_completion_tokens.or(m.max_output_tokens))
            .or_else(|| self.model.max_output_tokens())
    }

    fn count_tokens(
        &self,
        request: LanguageModelRequest,
        cx: &App,
    ) -> BoxFuture<'static, Result<u64>> {
        cx.background_spawn(async move {
            let messages = request
                .messages
                .into_iter()
                .map(|message| tiktoken_rs::ChatCompletionRequestMessage {
                    role: match message.role {
                        Role::User => "user".into(),
                        Role::Assistant => "assistant".into(),
                        Role::System => "system".into(),
                    },
                    content: Some(message.string_contents()),
                    name: None,
                    function_call: None,
                })
                .collect::<Vec<_>>();

            tiktoken_rs::num_tokens_from_messages("gpt-4", &messages).map(|tokens| tokens as u64)
        })
        .boxed()
    }

    fn stream_completion(
        &self,
        request: LanguageModelRequest,
        cx: &AsyncApp,
    ) -> BoxFuture<
        'static,
        Result<
            BoxStream<'static, Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
            LanguageModelCompletionError,
        >,
    > {
        let request = into_qwen(
            request,
            &self.model,
            self.settings_model.as_ref(),
            self.max_output_tokens(),
        );
        let stream = self.stream_completion(request, cx);

        async move {
            let mapper = QwenEventMapper::new();
            Ok(mapper.map_stream(stream.await?).boxed())
        }
        .boxed()
    }
}

pub fn into_qwen(
    request: LanguageModelRequest,
    model: &qwen::Model,
    settings_model: Option<&AvailableModel>,
    max_output_tokens: Option<u64>,
) -> qwen::Request {
    let mut messages = Vec::new();
    let mut current_reasoning: Option<String> = None;

    for message in request.messages {
        for content in message.content {
            match content {
                MessageContent::Text(text) => {
                    let should_add = if message.role == Role::User {
                        !text.trim().is_empty()
                    } else {
                        !text.is_empty()
                    };

                    if should_add {
                        messages.push(match message.role {
                            Role::User => qwen::RequestMessage::User { content: text },
                            Role::Assistant => qwen::RequestMessage::Assistant {
                                content: Some(text),
                                tool_calls: Vec::new(),
                                reasoning_content: current_reasoning.take(),
                            },
                            Role::System => qwen::RequestMessage::System { content: text },
                        });
                    }
                }
                MessageContent::Thinking { text, .. } => {
                    current_reasoning.get_or_insert_default().push_str(&text);
                }
                MessageContent::RedactedThinking(_) => {}
                MessageContent::Image(_) => {}
                MessageContent::ToolUse(tool_use) => {
                    let tool_call = qwen::ToolCall {
                        id: tool_use.id.to_string(),
                        content: qwen::ToolCallContent::Function {
                            function: qwen::FunctionContent {
                                name: tool_use.name.to_string(),
                                arguments: serde_json::to_string(&tool_use.input)
                                    .unwrap_or_default(),
                            },
                        },
                    };

                    if let Some(qwen::RequestMessage::Assistant { tool_calls, .. }) =
                        messages.last_mut()
                    {
                        tool_calls.push(tool_call);
                    } else {
                        messages.push(qwen::RequestMessage::Assistant {
                            content: None,
                            tool_calls: vec![tool_call],
                            reasoning_content: current_reasoning.take(),
                        });
                    }
                }
                MessageContent::ToolResult(tool_result) => {
                    match &tool_result.content {
                        LanguageModelToolResultContent::Text(text) => {
                            messages.push(qwen::RequestMessage::Tool {
                                content: text.to_string(),
                                tool_call_id: tool_result.tool_use_id.to_string(),
                            });
                        }
                        LanguageModelToolResultContent::Image(_) => {}
                    };
                }
            }
        }
    }

    // Only send enable_thinking when explicitly configured in settings.
    // Local servers (llama.cpp, vLLM) typically don't support this parameter
    // and will ignore it, causing the model to dump thinking into regular content.
    // The DashScope cloud API supports it natively.
    let explicit_thinking = settings_model.and_then(|m| m.enable_thinking);

    let thinking_budget = settings_model.and_then(|m| m.thinking_budget);
    let max_completion_tokens = settings_model.and_then(|m| m.max_completion_tokens);

    let parallel_tool_calls = settings_model.map(|m| m.capabilities.parallel_tool_calls);

    qwen::Request {
        model: model.id().to_string(),
        messages,
        stream: true,
        max_tokens: max_output_tokens,
        max_completion_tokens,
        temperature: request.temperature,
        tools: request
            .tools
            .into_iter()
            .map(|tool| qwen::ToolDefinition::Function {
                function: qwen::FunctionDefinition {
                    name: tool.name,
                    description: Some(tool.description),
                    parameters: Some(tool.input_schema),
                },
            })
            .collect(),
        parallel_tool_calls,
        stream_options: Some(qwen::StreamOptions {
            include_usage: true,
        }),
        enable_thinking: if explicit_thinking == Some(true) && request.thinking_allowed {
            Some(true)
        } else {
            None
        },
        thinking_budget: if explicit_thinking == Some(true) && request.thinking_allowed {
            thinking_budget
        } else {
            None
        },
    }
}

pub struct QwenEventMapper {
    tool_calls_by_index: HashMap<usize, RawToolCall>,
    sent_start_message: bool,
}

impl QwenEventMapper {
    pub fn new() -> Self {
        Self {
            tool_calls_by_index: HashMap::default(),
            sent_start_message: false,
        }
    }

    pub fn map_stream(
        mut self,
        events: Pin<Box<dyn Send + Stream<Item = Result<qwen::StreamResponse>>>>,
    ) -> impl Stream<Item = Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>
    {
        events.flat_map(move |event| {
            futures::stream::iter(match event {
                Ok(event) => self.map_event(event),
                Err(error) => vec![Err(LanguageModelCompletionError::from(error))],
            })
        })
    }

    pub fn map_event(
        &mut self,
        event: qwen::StreamResponse,
    ) -> Vec<Result<LanguageModelCompletionEvent, LanguageModelCompletionError>> {
        let mut events = Vec::new();

        // When stream_options.include_usage is true, the final chunk has
        // an empty choices array and only contains usage data.
        if event.choices.is_empty() {
            if let Some(usage) = event.usage {
                events.push(Ok(LanguageModelCompletionEvent::UsageUpdate(TokenUsage {
                    input_tokens: usage.prompt_tokens,
                    output_tokens: usage.completion_tokens,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                })));
            }
            return events;
        }

        let choice = &event.choices[0];

        let mut events = Vec::new();

        if !self.sent_start_message {
            self.sent_start_message = true;
            events.push(Ok(LanguageModelCompletionEvent::StartMessage {
                message_id: event.id.clone(),
            }));
        }

        if let Some(content) = choice.delta.content.clone()
            && !content.is_empty()
        {
            events.push(Ok(LanguageModelCompletionEvent::Text(content)));
        }

        if let Some(reasoning_content) = choice.delta.reasoning_content.clone() {
            if !reasoning_content.is_empty() {
                events.push(Ok(LanguageModelCompletionEvent::Thinking {
                    text: reasoning_content,
                    signature: None,
                }));
            }
        }

        if let Some(tool_calls) = choice.delta.tool_calls.as_ref() {
            for tool_call in tool_calls {
                let entry = self.tool_calls_by_index.entry(tool_call.index).or_default();

                if let Some(tool_id) = tool_call.id.clone() {
                    entry.id = tool_id;
                }

                if let Some(function) = tool_call.function.as_ref() {
                    if let Some(name) = function.name.clone() {
                        entry.name = name;
                    }

                    if let Some(arguments) = function.arguments.clone() {
                        entry.arguments.push_str(&arguments);
                    }
                }

                if !entry.id.is_empty() && !entry.name.is_empty() {
                    if let Ok(input) = serde_json::from_str::<serde_json::Value>(
                        &fix_streamed_json(&entry.arguments),
                    ) {
                        events.push(Ok(LanguageModelCompletionEvent::ToolUse(
                            LanguageModelToolUse {
                                id: entry.id.clone().into(),
                                name: entry.name.as_str().into(),
                                is_input_complete: false,
                                input,
                                raw_input: entry.arguments.clone(),
                                thought_signature: None,
                            },
                        )));
                    }
                }
            }
        }

        if let Some(usage) = event.usage {
            events.push(Ok(LanguageModelCompletionEvent::UsageUpdate(TokenUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            })));
        }

        match choice.finish_reason.as_deref() {
            Some("stop") => {
                events.push(Ok(LanguageModelCompletionEvent::Stop(StopReason::EndTurn)));
            }
            Some("tool_calls") => {
                events.extend(self.tool_calls_by_index.drain().map(|(_, tool_call)| {
                    match parse_tool_arguments(&tool_call.arguments) {
                        Ok(input) => Ok(LanguageModelCompletionEvent::ToolUse(
                            LanguageModelToolUse {
                                id: tool_call.id.clone().into(),
                                name: tool_call.name.as_str().into(),
                                is_input_complete: true,
                                input,
                                raw_input: tool_call.arguments.clone(),
                                thought_signature: None,
                            },
                        )),
                        Err(error) => Ok(LanguageModelCompletionEvent::ToolUseJsonParseError {
                            id: tool_call.id.clone().into(),
                            tool_name: tool_call.name.as_str().into(),
                            raw_input: tool_call.arguments.into(),
                            json_parse_error: error.to_string(),
                        }),
                    }
                }));

                events.push(Ok(LanguageModelCompletionEvent::Stop(StopReason::ToolUse)));
            }
            Some(stop_reason) => {
                log::error!("Unexpected Qwen stop_reason: {stop_reason:?}");
                events.push(Ok(LanguageModelCompletionEvent::Stop(StopReason::EndTurn)));
            }
            None => {}
        }

        events
    }
}

struct ConfigurationView {
    api_key_editor: Entity<InputField>,
    state: Entity<State>,
    load_credentials_task: Option<Task<()>>,
}

impl ConfigurationView {
    fn new(state: Entity<State>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let api_key_editor =
            cx.new(|cx| InputField::new(window, cx, "sk-00000000000000000000000000000000"));

        cx.observe(&state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let load_credentials_task = Some(cx.spawn({
            let state = state.clone();
            async move |this, cx| {
                if let Some(task) = Some(state.update(cx, |state, cx| state.authenticate(cx))) {
                    let _ = task.await;
                }

                this.update(cx, |this, cx| {
                    this.load_credentials_task = None;
                    cx.notify();
                })
                .log_err();
            }
        }));

        Self {
            api_key_editor,
            state,
            load_credentials_task,
        }
    }

    fn save_api_key(&mut self, _: &menu::Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_editor.read(cx).text(cx).trim().to_string();
        if api_key.is_empty() {
            return;
        }

        let state = self.state.clone();
        cx.spawn(async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(Some(api_key), cx))
                .await
        })
        .detach_and_log_err(cx);
    }

    fn reset_api_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.api_key_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));

        let state = self.state.clone();
        cx.spawn(async move |_, cx| {
            state
                .update(cx, |state, cx| state.set_api_key(None, cx))
                .await
        })
        .detach_and_log_err(cx);
    }

    fn should_render_editor(&self, cx: &mut Context<Self>) -> bool {
        !self.state.read(cx).is_authenticated()
    }
}

impl Render for ConfigurationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let env_var_set = self.state.read(cx).api_key_state.is_from_env_var();
        let configured_card_label = if env_var_set {
            format!("API key set in {API_KEY_ENV_VAR_NAME} environment variable")
        } else {
            let api_url = QwenLanguageModelProvider::api_url(cx);
            if api_url == QWEN_API_URL {
                "API key configured".to_string()
            } else {
                format!("API key configured for {}", api_url)
            }
        };

        if self.load_credentials_task.is_some() {
            div()
                .child(Label::new("Loading credentials..."))
                .into_any_element()
        } else if self.should_render_editor(cx) {
            v_flex()
                .size_full()
                .on_action(cx.listener(Self::save_api_key))
                .child(Label::new(
                    "To use Qwen models in Zed, you need an API key. \
                     You can also point this at a local server (vLLM, Ollama, etc.) \
                     by setting the API URL in settings.",
                ))
                .child(
                    List::new()
                        .child(
                            ListBulletItem::new("")
                                .child(Label::new("Get your API key from"))
                                .child(ButtonLink::new(
                                    "Alibaba Cloud Model Studio",
                                    "https://www.alibabacloud.com/help/en/model-studio/get-api-key",
                                )),
                        )
                        .child(ListBulletItem::new(
                            "Paste your API key below and hit enter to start using Qwen",
                        )),
                )
                .child(self.api_key_editor.clone())
                .child(
                    Label::new(format!(
                        "You can also set the {API_KEY_ENV_VAR_NAME} environment variable and restart Zed."
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
                )
                .into_any_element()
        } else {
            ConfiguredApiCard::new(configured_card_label)
                .disabled(env_var_set)
                .on_click(cx.listener(|this, _, window, cx| this.reset_api_key(window, cx)))
                .into_any_element()
        }
    }
}
