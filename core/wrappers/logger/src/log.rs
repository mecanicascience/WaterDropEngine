use std::collections::BTreeMap;
use std::io::Write;

use tracing::field::Field;
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
use tracing_subscriber::Layer;


/// Struct to store custom fields in a span.
#[derive(Debug)]
struct CustomFieldStorage(BTreeMap<String, serde_json::Value>);

/// Struct to visit all of the fields in a message event.
struct JsonVisitor<'a>(&'a mut BTreeMap<String, serde_json::Value>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
}


pub struct LoggerLayer {
    file_name: String,
}

impl LoggerLayer {
    pub fn new(file_name: &str) -> Self {
        // Create the file if it doesn't exist
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .unwrap();

        // Clear the file
        std::fs::write(file_name, "").unwrap();

        Self {
            file_name: file_name.to_string(),
        }
    }
}

impl<S> Layer<S> for LoggerLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // A new span has been created
        let span = ctx.span(id).unwrap();

        // Get the fields
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        attrs.record(&mut visitor);

        // Store the fields in the span
        let storage = CustomFieldStorage(fields);
        let mut extensions = span.extensions_mut();
        extensions.insert(storage);
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Get the span whose data is being recorded
        let span = ctx.span(id).unwrap();

        // Get a mutable reference to the data we created in new_span
        let mut extensions_mut = span.extensions_mut();
        let custom_field_storage: &mut CustomFieldStorage =
            extensions_mut.get_mut::<CustomFieldStorage>().unwrap();
        let json_data: &mut BTreeMap<String, serde_json::Value> = &mut custom_field_storage.0;

        // Add the new data to the JSON object
        let mut visitor = JsonVisitor(json_data);
        values.record(&mut visitor);
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // If message starts with "log", discard it
        if event.metadata().target().starts_with("log") || event.metadata().target().starts_with("polling::epoll") {
            return;
        }

        // Read the parent spans of the event
        let scope_wrapped = ctx.event_scope(event);
        let scope = match scope_wrapped {
            Some(scope) => Some(scope),
            None => None,
        };
        let mut spans = vec![];
        if let Some(scope) = scope {
            for span in scope.from_root() {
                // Read the stored fields
                let extensions = span.extensions();
                let storage = extensions.get::<CustomFieldStorage>().unwrap();

                // As JSON
                let field_data: &BTreeMap<String, serde_json::Value> = &storage.0;
                spans.push(serde_json::json!({
                    "name": span.metadata().name(),
                    "target": span.metadata().target(),
                    "level": format!("{:?}", match span.metadata().level() {
                        &tracing::Level::TRACE => "TRACE",
                        &tracing::Level::DEBUG => "DEBUG",
                        &tracing::Level::INFO => "INFO",
                        &tracing::Level::WARN => "WARN",
                        &tracing::Level::ERROR => "ERROR",
                    }),
                    "fields": field_data,
                    "callsite": format!("{:?}", span.metadata().callsite())
                }));
            }
        }

        // Convert the values of the message into a JSON object
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        // Write the message in JSON
        let output = serde_json::json!({
            "name": event.metadata().name(),
            "target": event.metadata().target(),
            "level": format!("{:?}", event.metadata().level().as_str()),
            "callsite": format!("{:?}", event.metadata().callsite()),
            "fields": fields,
            "spans": spans,
        });
        
        // Format the other fields
        let mut other_fields = String::new();
        for (key, value) in output["fields"].as_object().unwrap() {
            if key != "message" {
                other_fields.push_str(&format!("{}: {}, ", key, value));
            }
        }
        other_fields.pop();
        if other_fields.len() > 0 {
            other_fields = "{ ".to_string() + &other_fields + " }";
        }

        // Format the message for the console
        let date = chrono::Local::now().format("%H:%M:%S:%3f");
        let message_formated = match output["level"].as_str().unwrap() {
            "\"TRACE\"" => format!("\x1b[30m{}\x1b[0m \x1b[30m[{}]\t{} : {} {}\x1b[0m", date, "TRACE", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
            "\"DEBUG\"" => format!("\x1b[30m{}\x1b[0m \x1b[34m[{}]\t{} : {} {}\x1b[0m", date, "DEBUG", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
            "\"INFO\"" => format!("\x1b[30m{}\x1b[0m \x1b[32m[{}]\t{}  : {} {}\x1b[0m", date, "INFO", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
            "\"WARN\"" => format!("\x1b[30m{}\x1b[0m \x1b[33m[{}]\t{}  : {} {}\x1b[0m", date, "WARN", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
            "\"ERROR\"" => format!("\x1b[30m{}\x1b[0m \x1b[31m[{}]\t{} : {} {}\x1b[0m", date, "ERROR", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
            _ => format!("{} [{}]\t{} : {} {}", date, "?????", output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap(), other_fields),
        };
        println!("{}", message_formated);

        // Format the message raw
        let message_raw = format!("{}\t[{}]\t{}\t:\t{}", date, output["level"].as_str().unwrap(),
            output["target"].as_str().unwrap(), output["fields"]["message"].as_str().unwrap());

        // Write to file sync
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_name)
            .unwrap();
        file.write_all(format!("{}\n", message_raw).as_bytes())
            .unwrap();
    }
}

