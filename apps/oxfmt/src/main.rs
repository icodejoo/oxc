// Pure Rust CLI entry point.
// Supports the basic `Cli` mode plus `--lsp` and `--stdin-filepath`,
// but no Prettier delegation, no JS/TS config files, and no `--init` / `--migrate`.
// For full featured JS CLI entry point, see `run_cli()` exported by `main_napi.rs`.

#[cfg(not(feature = "napi"))]
mod pure {
    use std::sync::Arc;

    use oxfmt::{
        ExternalServices,
        cli::{
            CliRunResult, Mode, StdinRunner, WalkRunner, format_command, init_rayon, init_tracing,
        },
        lsp::run_lsp,
    };

    /// JS/TS config files need the Node.js CLI, so the loader always reports that.
    #[expect(clippy::needless_pass_by_value)]
    fn unsupported_js_config(path: String) -> Result<serde_json::Value, String> {
        Err(format!(
            "JS/TS config file ({path}) is not supported in pure Rust CLI.\nUse JSON/JSONC instead."
        ))
    }

    pub async fn run() -> CliRunResult {
        let command = format_command().run();

        init_tracing();
        match command.mode {
            Mode::Lsp => {
                run_lsp(Arc::new(unsupported_js_config), ExternalServices).await;
                CliRunResult::None
            }
            Mode::Stdin(_) => {
                StdinRunner::new(command, Arc::new(unsupported_js_config), ExternalServices).run()
            }
            Mode::Cli(_) => {
                init_rayon(command.runtime_options.threads);
                WalkRunner::new(command).run()
            }
        }
    }
}

#[cfg(not(feature = "napi"))]
#[tokio::main]
async fn main() -> oxfmt::cli::CliRunResult {
    pure::run().await
}

// With the `napi` feature the JS entry point is used instead, this binary has nothing to run.
#[cfg(feature = "napi")]
fn main() {}
