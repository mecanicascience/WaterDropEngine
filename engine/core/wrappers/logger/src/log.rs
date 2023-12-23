// Alias for tracing macros
pub use tracing::trace_span as trace_span;
pub use tracing::debug_span as debug_span;
pub use tracing::info_span as info_span;
pub use tracing::warn_span as warn_span;
pub use tracing::error_span as error_span;

// Alias for logger macros
pub use tracing::trace as trace;
pub use tracing::debug as debug;
pub use tracing::info as info;
pub use tracing::warn as warn;
pub use tracing::error as error;

// Export macro
pub use tracing::instrument as logger;
