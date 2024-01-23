use std::{collections::BTreeMap, io::{Read, Write}};
use regex::Regex;
use tracing::{field::Field, span};
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

impl JsonVisitor<'_> {
    fn set_thread_id(&mut self, thread_id: u64) {
        self.0.insert("thread_id".to_string(), serde_json::json!(thread_id));
    }
    fn set_start(&mut self, start: u128) {
        // Convert to string to avoid overflow
        self.0.insert("start".to_string(), serde_json::json!(start.to_string()));
    }
}


pub struct TracingLayer {
    file_name: String,
}

impl TracingLayer {
    pub fn new(file_name: &str) -> Self {
        // Check if production
        if !cfg!(debug_assertions) {
            return Self {
                file_name: "".to_string(),
            };
        }

        // Create the file if it doesn't exist
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .unwrap();

        // Write the header
        std::fs::write(file_name, "{\"traceEvents\":[\n").unwrap();

        Self {
            file_name: file_name.to_string(),
        }
    }

    pub fn close(file_name: &String) {
        // Check if production
        if !cfg!(debug_assertions) {
            return;
        }

        // Wait for all async writes to finish
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Remove the last comma
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(file_name)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents.pop();
        contents.pop();
        std::fs::write(file_name, contents).unwrap();

        // Write the footer
        std::fs::OpenOptions::new()
            .append(true)
            .open(file_name)
            .unwrap()
            .write_all(b"\n]}\n")
            .unwrap();
    }
}


impl<S> Layer<S> for TracingLayer
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

        // Set the thread ID
        let thread_str = format!("{:?}", std::thread::current().id());
        let thread_id = Regex::new(r"ThreadId\((\d+)\)").unwrap()
            .captures(&thread_str).unwrap()[1]
            .parse::<u64>().unwrap();
        visitor.set_thread_id(thread_id);

        // Set the time
        visitor.set_start(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());

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

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Get span storage
        let span = ctx.span(&id).unwrap();
        let extensions = span.extensions();
        let storage = extensions.get::<CustomFieldStorage>().unwrap();

        // Get duration
        let start = storage.0["start"].as_str().unwrap().parse::<u128>().unwrap();
        let dur = ((
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            - start
        ) / 1000) as u128;

        // Create JSON object
        let data = serde_json::json!({
            "name": format!("{}::{}", span.metadata().target(), span.metadata().name()),
            "cat": match span.metadata().level() {
                &tracing::Level::TRACE => "TRACE",
                &tracing::Level::DEBUG => "DEBUG",
                &tracing::Level::INFO => "INFO",
                &tracing::Level::WARN => "WARN",
                &tracing::Level::ERROR => "ERROR",
            },
            "ph": "X",
            "pid": 1,
            "tid": storage.0["thread_id"],
            "ts": (start / 1000) as u128,
            "dur": dur,
            "args": storage.0,
        });

        // // Write to file async using tokio
        // let file_name = self.file_name.clone();
        // tokio::spawn(async move {
        //     let mut file = tokio::fs::OpenOptions::new()
        //         .create(true)
        //         .append(true)
        //         .open(file_name)
        //         .await
        //         .unwrap();
        //     file.write_all(format!("{},\n", data).as_bytes())
        //         .await
        //         .unwrap();
        // });
        // Write to file sync
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_name)
            .unwrap();
        file.write_all(format!("{},\n", data).as_bytes())
            .unwrap();
    }
}
