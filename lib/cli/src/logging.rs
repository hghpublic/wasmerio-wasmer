//! Logging functions for the debug feature.

use is_terminal::IsTerminal;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const WHITELISTED_LOG_TARGETS: &[&str] = &["wasmer", "wasmer_wasix", "virtual_fs"];

/// Control the output generated by the CLI.
#[derive(Debug, Default, Clone, PartialEq, clap::Parser)]
pub struct Output {
    /// Generate verbose output (repeat for more verbosity)
    #[clap(short, long, action = clap::ArgAction::Count, global = true, conflicts_with = "quiet")]
    pub verbose: u8,
    /// Do not print progress messages.
    #[clap(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    /// The format to use when generating logs.
    #[clap(long, global = true, env, default_value = "text")]
    pub log_format: LogFormat,
    /// When to display colored output.
    #[clap(long, default_value_t = clap::ColorChoice::Auto, global = true)]
    pub color: clap::ColorChoice,
}

impl Output {
    /// Has the `--verbose` flag been set?
    pub fn is_verbose(&self) -> bool {
        self.verbose > 0
    }

    /// Initialize logging based on the `$RUST_LOG` environment variable and
    /// command-line flags.
    pub fn initialize_logging(&self) {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .with_ansi(self.should_emit_colors())
            .with_thread_ids(true)
            .with_writer(std::io::stderr);

        let filter_layer = self.log_filter();

        match self.log_format {
            LogFormat::Text => tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer.compact().with_target(true))
                .init(),
            LogFormat::Json => tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer.json().with_target(true))
                .init(),
        }
    }

    fn log_filter(&self) -> EnvFilter {
        let default_filters = [
            LevelFilter::OFF,
            LevelFilter::WARN,
            LevelFilter::INFO,
            LevelFilter::DEBUG,
        ];

        // First, we set up the default log level.
        let default_level = default_filters
            .get(self.verbose as usize)
            .copied()
            .unwrap_or(LevelFilter::TRACE);
        let mut filter = EnvFilter::builder()
            .with_default_directive(default_level.into())
            .from_env_lossy();

        // Next we add level-specific directives, where verbosity=0 means don't
        // override anything. Note that these are shifted one level up so we'll
        // get something like RUST_LOG="warn,wasmer_wasix=info"
        let specific_filters = [LevelFilter::WARN, LevelFilter::INFO, LevelFilter::DEBUG];
        if self.verbose > 0 {
            let level = specific_filters
                .get(self.verbose as usize)
                .copied()
                .unwrap_or(LevelFilter::TRACE);

            for target in WHITELISTED_LOG_TARGETS {
                let directive = format!("{target}={level}").parse().unwrap();
                filter = filter.add_directive(directive);
            }
        }

        filter
    }

    /// Check whether we should emit ANSI escape codes for log formatting.
    ///
    /// The `tracing-subscriber` crate doesn't have native support for
    /// "--color=always|never|auto", so we implement a poor man's version.
    ///
    /// For more, see https://github.com/tokio-rs/tracing/issues/2388
    fn should_emit_colors(&self) -> bool {
        match self.color {
            clap::ColorChoice::Auto => std::io::stderr().is_terminal(),
            clap::ColorChoice::Always => true,
            clap::ColorChoice::Never => false,
        }
    }

    /// Get the draw target to be used with the `indicatif` crate.
    ///
    /// Progress indicators won't draw anything if the user passed the `--quiet`
    /// flag.
    pub fn draw_target(&self) -> indicatif::ProgressDrawTarget {
        if self.quiet {
            return indicatif::ProgressDrawTarget::hidden();
        }

        indicatif::ProgressDrawTarget::stderr()
    }
}

/// The format used when generating logs.
#[derive(Debug, Default, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum LogFormat {
    /// Human-readable logs.
    #[default]
    Text,
    /// Machine-readable logs.
    Json,
}
