use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// Compose multiple layers into a `tracing`'s subscriber.
//
// # Implementation Notes
//
// We are using `impl Subscriber` as return type to avoid having to
// spell out the acual type of the returned subscriber, which is complex.
// We need to explicitly call out that the returned subscriber is `Send` and `Sync`
// to make it possible to pass it to `init_subscriber` later on.
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Sync + Send {
    // Fallback to printing all logs at info-level or above if RUST_LOG env variable has not been set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Output log records in bunyan-compatible JSON format.
    let formatting_layer = BunyanFormattingLayer::new(
        name,
        // Output formatted log to stdout.
        std::io::stdout,
    )
    .skip_fields(vec!["line", "file", "target"].into_iter())
    .expect("One of the specified fields cannot be skipped");

    // `Registry` implements subscriber and provides span storage.
    Registry::default()
        // `layer::SubscriberExt` trait adding a `with(Layer)` combinator to `Subscriber`s.
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    // `set_global_default` is used to specify which subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}
