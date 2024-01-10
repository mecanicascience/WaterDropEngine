use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::FmtSubscriber;

/// Log levels.
pub enum LEVEL {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

/// Create a new logger instance.
/// This must be called once before any other logging function.
/// 
/// # Arguments
/// 
/// * `level` - Maximum level of the logging system.
pub fn create_logger(level: LEVEL) {
    // Build a tracing subscriber that writes to stdout
    let subscriber = FmtSubscriber::builder()
        .with_max_level(match level {
            LEVEL::TRACE => Level::TRACE,
            LEVEL::DEBUG => Level::DEBUG,
            LEVEL::INFO => Level::INFO,
            LEVEL::WARN => Level::WARN,
            LEVEL::ERROR => Level::ERROR,
        })
        .finish();

    // Set the global subscriber as the default for this thread
    set_global_default(subscriber)
        .expect("Setting default logger subscriber failed.");
}
