mod config;
pub mod embed;
mod format;
mod global_ignore;
mod language;
pub mod options;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_services;

pub use config::{ConfigResolver, ConfigScopes, ResolveOutcome, config_discovery};
#[cfg(feature = "napi")]
pub use config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader, resolve_for_api};
pub use format::{FormatPlan, FormatResult, SourceFormatter};
pub use global_ignore::{build_global_ignore_matchers, is_ignored, resolve_ignore_paths};
pub use language::Language;
#[cfg(feature = "napi")]
pub use support::classify_file;

/// Stand-in for the napi config loader callback type in the pure Rust build,
/// where it only travels through the LSP / Stdin entry points and is never called.
#[cfg(not(feature = "napi"))]
pub type JsConfigLoaderCb =
    std::sync::Arc<dyn Fn(String) -> Result<serde_json::Value, String> + Send + Sync>;
#[cfg(not(feature = "napi"))]
mod external_services_stub;
#[cfg(not(feature = "napi"))]
pub use external_services_stub::ExternalServices;
#[cfg(feature = "napi")]
pub use external_services::{
    ExternalServices, JsFormatEmbeddedDocCb, JsFormatFileCb, JsInitExternalServicesCb,
    JsSortTailwindClassesCb,
};
