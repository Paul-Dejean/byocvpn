use std::{
    env,
    io::{IsTerminal, stderr},
};

use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan};

const LOG_STYLE_VARIABLE: &str = "RUST_LOG_STYLE";

#[derive(Debug, Clone, Copy)]
pub struct LoggingFormat {
    pub show_timestamps: bool,
    pub show_targets: bool,
}

impl LoggingFormat {
    pub const DETAILED: Self = Self {
        show_timestamps: true,
        show_targets: true,
    };
    pub const MESSAGE_ONLY: Self = Self {
        show_timestamps: false,
        show_targets: false,
    };
}

pub fn initialize_logging(default_directives: &str, format: LoggingFormat) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directives));
    let subscriber = fmt()
        .with_env_filter(filter)
        .with_writer(stderr)
        .with_ansi(should_use_colors())
        .with_span_events(FmtSpan::CLOSE)
        .with_target(format.show_targets);
    if format.show_timestamps {
        subscriber.init();
    } else {
        subscriber.without_time().init();
    }
}

fn should_use_colors() -> bool {
    match env::var(LOG_STYLE_VARIABLE).as_deref() {
        Ok("always") => true,
        Ok("never") => false,
        _ => stderr().is_terminal(),
    }
}
