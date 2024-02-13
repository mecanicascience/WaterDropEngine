use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{LoggerLayer, TracingLayer};

pub struct Logger {
    tracing_file_name: String,
}

impl Logger {
    /// Create a new logger instance.
    /// This must be called once before any other logging function.
    /// 
    /// # Arguments
    /// 
    /// * `log_file_name` - The name of the file to write the log data to.
    /// * `tracing_file_name` - The name of the file to write the tracing data to.
    pub fn new(log_file_name: &str ,tracing_file_name: &str) -> Self {
        // Custom logger layer
        let logger_layer = LoggerLayer::new(log_file_name);

        #[cfg(feature = "tracing")]
        {
            // Custom tracing layer
            let tracing_layer = TracingLayer::new(tracing_file_name);
            tracing_subscriber::registry()
                .with(tracing_layer)
                .with(logger_layer)
                .init();
        }

        #[cfg(not(feature = "tracing"))]
        tracing_subscriber::registry()
            .with(logger_layer)
            .init();

        Self {
            tracing_file_name: tracing_file_name.to_string(),
        }
    }

    /// Close the logger instance.
    /// This must be called once after all other logging functions.
    pub fn close(&self) {
        // Write the footer
        TracingLayer::close(&self.tracing_file_name);
    }
}
