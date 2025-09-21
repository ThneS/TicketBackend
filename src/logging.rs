use std::{env, path::Path, sync::OnceLock};
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, layer::SubscriberExt,
    util::SubscriberInitExt,
};

static INITED: OnceLock<()> = OnceLock::new();

fn env_filter() -> EnvFilter {
    // Default to info if RUST_LOG unset
    let default_directives = "info";
    match EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(_) => EnvFilter::new(default_directives),
    }
}

/// Initialize global tracing subscriber exactly once.
///
/// Env vars:
/// - RUST_LOG: env filter (e.g. "backend=debug,axum=info")
/// - LOG_FORMAT: "pretty" (default in dev) or "json"
/// - LOG_FILE: if set, also write logs to this file with rotation by size (daily not included)
/// - LOG_ANSI: "0" to disable ANSI colors
pub fn init() {
    if INITED.get().is_some() {
        return;
    }

    let filter = env_filter();
    let fmt_layer_builder = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(env::var("LOG_ANSI").map_or(true, |v| v != "0"));

    let format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".into());

    // Console formatting
    let console_layer = if format.eq_ignore_ascii_case("json") {
        fmt_layer_builder.json().boxed()
    } else {
        fmt_layer_builder.pretty().boxed()
    };

    let base = Registry::default().with(filter).with(console_layer);
    if let Ok(file_var) = env::var("LOG_FILE") {
        if !file_var.is_empty() {
            // Derive dir/file from LOG_FILE and optional LOG_DIR
            let p = Path::new(&file_var);
            let file_name = p.file_name().and_then(|s| s.to_str());
            let (dir, file) = if let Some(fname) = file_name {
                let parent = p.parent().and_then(|pp| {
                    let s = pp.to_string_lossy();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.into_owned())
                    }
                });
                let dir = parent
                    .or_else(|| env::var("LOG_DIR").ok())
                    .unwrap_or_else(|| ".".into());
                (dir, fname.to_string())
            } else {
                let dir = env::var("LOG_DIR").unwrap_or_else(|_| ".".into());
                (dir, file_var)
            };

            // Time-based rotation: never|daily|hourly|minutely
            let rotation =
                env::var("LOG_ROTATION").unwrap_or_else(|_| "never".into());
            let file_appender = match rotation.to_lowercase().as_str() {
                "daily" => tracing_appender::rolling::daily(&dir, &file),
                "hourly" => tracing_appender::rolling::hourly(&dir, &file),
                "minutely" => tracing_appender::rolling::minutely(&dir, &file),
                _ => tracing_appender::rolling::never(&dir, &file),
            };

            let (non_blocking, _guard) =
                tracing_appender::non_blocking(file_appender);
            // Keep guard alive by leaking; process-lifetime static. Acceptable for a long-running service.
            Box::leak(Box::new(_guard));
            let file_layer = fmt::layer()
                .with_ansi(false)
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .json()
                .with_writer(non_blocking)
                .boxed();

            base.with(file_layer).init();
        } else {
            base.init();
        }
    } else {
        base.init();
    }

    let _ = INITED.set(());
}

/// Convenience re-export of common macros
pub use tracing::{debug, error, info, instrument, trace, warn};
