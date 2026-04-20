use std::sync::Arc;

use collections::HashMap;
use settings::RegisterSetting;

#[cfg(feature = "anthropic")]
use crate::provider::anthropic::AnthropicSettings;
#[cfg(feature = "bedrock")]
use crate::provider::bedrock::AmazonBedrockSettings;
#[cfg(feature = "cloud")]
use crate::provider::cloud::ZedDotDevSettings;
#[cfg(feature = "deepseek")]
use crate::provider::deepseek::DeepSeekSettings;
#[cfg(feature = "google")]
use crate::provider::google::GoogleSettings;
#[cfg(feature = "lmstudio")]
use crate::provider::lmstudio::LmStudioSettings;
#[cfg(feature = "mistral")]
use crate::provider::mistral::MistralSettings;
#[cfg(feature = "ollama")]
use crate::provider::ollama::OllamaSettings;
use crate::provider::open_ai::OpenAiSettings;
use crate::provider::open_ai_compatible::OpenAiCompatibleSettings;
#[cfg(feature = "open_router")]
use crate::provider::open_router::OpenRouterSettings;
#[cfg(feature = "opencode")]
use crate::provider::opencode::OpenCodeSettings;
use crate::provider::qwen::QwenSettings;
#[cfg(feature = "vercel")]
use crate::provider::vercel::VercelSettings;
#[cfg(feature = "vercel_ai_gateway")]
use crate::provider::vercel_ai_gateway::VercelAiGatewaySettings;
#[cfg(feature = "x_ai")]
use crate::provider::x_ai::XAiSettings;

#[derive(Debug, RegisterSetting)]
pub struct AllLanguageModelSettings {
    #[cfg(feature = "anthropic")]
    pub anthropic: AnthropicSettings,
    #[cfg(feature = "bedrock")]
    pub bedrock: AmazonBedrockSettings,
    #[cfg(feature = "deepseek")]
    pub deepseek: DeepSeekSettings,
    #[cfg(feature = "google")]
    pub google: GoogleSettings,
    #[cfg(feature = "lmstudio")]
    pub lmstudio: LmStudioSettings,
    #[cfg(feature = "mistral")]
    pub mistral: MistralSettings,
    #[cfg(feature = "ollama")]
    pub ollama: OllamaSettings,
    #[cfg(feature = "opencode")]
    pub opencode: OpenCodeSettings,
    #[cfg(feature = "open_router")]
    pub open_router: OpenRouterSettings,
    pub openai: OpenAiSettings,
    pub openai_compatible: HashMap<Arc<str>, OpenAiCompatibleSettings>,
    pub qwen: QwenSettings,
    #[cfg(feature = "vercel")]
    pub vercel: VercelSettings,
    #[cfg(feature = "vercel_ai_gateway")]
    pub vercel_ai_gateway: VercelAiGatewaySettings,
    #[cfg(feature = "x_ai")]
    pub x_ai: XAiSettings,
    #[cfg(feature = "cloud")]
    pub zed_dot_dev: ZedDotDevSettings,
}

impl settings::Settings for AllLanguageModelSettings {
    const PRESERVED_KEYS: Option<&'static [&'static str]> = Some(&["version"]);

    fn from_settings(content: &settings::SettingsContent) -> Self {
        let language_models = content.language_models.clone().unwrap();

        #[cfg(feature = "anthropic")]
        let anthropic = language_models.anthropic.unwrap();
        #[cfg(feature = "bedrock")]
        let bedrock = language_models.bedrock.unwrap();
        #[cfg(feature = "deepseek")]
        let deepseek = language_models.deepseek.unwrap();
        #[cfg(feature = "google")]
        let google = language_models.google.unwrap();
        #[cfg(feature = "lmstudio")]
        let lmstudio = language_models.lmstudio.unwrap();
        #[cfg(feature = "mistral")]
        let mistral = language_models.mistral.unwrap();
        #[cfg(feature = "ollama")]
        let ollama = language_models.ollama.unwrap();
        #[cfg(feature = "opencode")]
        let opencode = language_models.opencode.unwrap();
        #[cfg(feature = "open_router")]
        let open_router = language_models.open_router.unwrap();
        let openai = language_models.openai.unwrap();
        let openai_compatible = language_models.openai_compatible.unwrap();
        let qwen = language_models.qwen.unwrap();
        #[cfg(feature = "vercel")]
        let vercel = language_models.vercel.unwrap();
        #[cfg(feature = "vercel_ai_gateway")]
        let vercel_ai_gateway = language_models.vercel_ai_gateway.unwrap();
        #[cfg(feature = "x_ai")]
        let x_ai = language_models.x_ai.unwrap();
        #[cfg(feature = "cloud")]
        let zed_dot_dev = language_models.zed_dot_dev.unwrap();

        Self {
            #[cfg(feature = "anthropic")]
            anthropic: AnthropicSettings {
                api_url: anthropic.api_url.unwrap(),
                available_models: anthropic.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "bedrock")]
            bedrock: AmazonBedrockSettings {
                available_models: bedrock.available_models.unwrap_or_default(),
                region: bedrock.region,
                endpoint: bedrock.endpoint_url, // todo(should be api_url)
                profile_name: bedrock.profile,
                role_arn: None, // todo(was never a setting for this...)
                authentication_method: bedrock.authentication_method.map(Into::into),
                allow_global: bedrock.allow_global,
                allow_extended_context: bedrock.allow_extended_context,
            },
            #[cfg(feature = "deepseek")]
            deepseek: DeepSeekSettings {
                api_url: deepseek.api_url.unwrap(),
                available_models: deepseek.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "google")]
            google: GoogleSettings {
                api_url: google.api_url.unwrap(),
                available_models: google.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "lmstudio")]
            lmstudio: LmStudioSettings {
                api_url: lmstudio.api_url.unwrap(),
                available_models: lmstudio.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "mistral")]
            mistral: MistralSettings {
                api_url: mistral.api_url.unwrap(),
                available_models: mistral.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "ollama")]
            ollama: OllamaSettings {
                api_url: ollama.api_url.unwrap(),
                auto_discover: ollama.auto_discover.unwrap_or(true),
                available_models: ollama.available_models.unwrap_or_default(),
                context_window: ollama.context_window,
            },
            #[cfg(feature = "opencode")]
            opencode: OpenCodeSettings {
                api_url: opencode.api_url.unwrap(),
                available_models: opencode.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "open_router")]
            open_router: OpenRouterSettings {
                api_url: open_router.api_url.unwrap(),
                available_models: open_router.available_models.unwrap_or_default(),
            },
            openai: OpenAiSettings {
                api_url: openai.api_url.unwrap(),
                available_models: openai.available_models.unwrap_or_default(),
            },
            openai_compatible: openai_compatible
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        OpenAiCompatibleSettings {
                            api_url: value.api_url,
                            available_models: value.available_models,
                        },
                    )
                })
                .collect(),
            qwen: QwenSettings {
                api_url: qwen.api_url.unwrap(),
                available_models: qwen.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "vercel")]
            vercel: VercelSettings {
                api_url: vercel.api_url.unwrap(),
                available_models: vercel.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "vercel_ai_gateway")]
            vercel_ai_gateway: VercelAiGatewaySettings {
                api_url: vercel_ai_gateway.api_url.unwrap(),
                available_models: vercel_ai_gateway.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "x_ai")]
            x_ai: XAiSettings {
                api_url: x_ai.api_url.unwrap(),
                available_models: x_ai.available_models.unwrap_or_default(),
            },
            #[cfg(feature = "cloud")]
            zed_dot_dev: ZedDotDevSettings {
                available_models: zed_dot_dev.available_models.unwrap_or_default(),
            },
        }
    }
}
