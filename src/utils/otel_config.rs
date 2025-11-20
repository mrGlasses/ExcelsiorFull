use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, TracerProvider};
use opentelemetry_sdk::{runtime, Resource};
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_telemetry() -> Result<TracerProvider> {
    // Get OTLP endpoint from the environment or use default
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    // Service name for identifying this microservice in Grafana
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "excelsior".to_string());

    // Build resource attributes for this service
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        KeyValue::new(
            "deployment.environment",
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()),
        ),
    ]);

    // Configure OTLP exporter with proper timeout and batching
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build OTLP exporter - check OTEL_EXPORTER_OTLP_ENDPOINT");

    // Create tracer provider with batch processor for better performance
    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .with_resource(resource)
        .with_sampler(Sampler::AlwaysOn) // For production, consider ParentBased(TraceIdRatioBased(0.1))
        .with_id_generator(RandomIdGenerator::default())
        .build();

    // Set a global tracer provider for the application
    global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}

/// Setup complete tracing infrastructure with OpenTelemetry layer
/// This replaces your current setup_tracing() function
pub fn setup_tracing_with_otel() {
    // Initialize OpenTelemetry tracer provider
    let tracer_provider = init_telemetry()
        .expect("Failed to initialize OpenTelemetry - cannot proceed without telemetry");

    // Get tracer for this service
    let tracer = tracer_provider.tracer("excelsior-ms1");

    // Create OpenTelemetry tracing layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create fmt layer for local console output (same as before)
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    // Build the complete subscriber with all layers
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt_layer)
        .with(otel_layer)
        .init();
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}
