use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{info, warn};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

pub fn init_telemetry() -> Result<SdkTracerProvider> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "excelsior".to_string());

    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new(
                "deployment.environment",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()),
            ),
        ])
        .build();

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build OTLP exporter - check OTEL_EXPORTER_OTLP_ENDPOINT");

    // Metrics go to the same collector, with the same service name (OT-2).
    init_metrics(&otlp_endpoint, resource.clone());

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .with_sampler(Sampler::AlwaysOn) // For production, consider ParentBased(TraceIdRatioBased(0.1))
        .with_id_generator(RandomIdGenerator::default())
        .build();

    global::set_tracer_provider(tracer_provider.clone());
    // Only the first call in the process wins (OnceLock); shutdown_telemetry() below
    // depends on this to ever have a provider to shut down.
    let _ = TRACER_PROVIDER.set(tracer_provider.clone());

    Ok(tracer_provider)
}

// OT-2: counters and other metrics, exported over OTLP like the traces. The periodic
// reader sends every 60s; OTEL_METRIC_EXPORT_INTERVAL (in milliseconds) changes that.
fn init_metrics(otlp_endpoint: &str, resource: Resource) {
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build OTLP metric exporter - check OTEL_EXPORTER_OTLP_ENDPOINT");

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(resource)
        .build();

    global::set_meter_provider(meter_provider.clone());
    // Same rule as the tracer: the first one is kept, so shutdown_telemetry() can flush it.
    let _ = METER_PROVIDER.set(meter_provider);
}

pub fn setup_tracing_with_otel() {
    let tracer_provider = init_telemetry()
        .expect("Failed to initialize OpenTelemetry - cannot proceed without telemetry");

    let tracer = tracer_provider.tracer("excelsior");

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    // try_init (not init) because in the test binary, several tests call this via
    // service_starter(); a global subscriber can only be set once per process, and any
    // call after the first is expected to no-op rather than panic.
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt_layer)
        .with(otel_layer)
        .try_init();
}

#[cfg(test)]
pub(crate) fn is_tracer_provider_set() -> bool {
    TRACER_PROVIDER.get().is_some()
}

#[cfg(test)]
pub(crate) fn is_meter_provider_set() -> bool {
    METER_PROVIDER.get().is_some()
}

// Sends what is still waiting (spans go out every few seconds, metrics every minute),
// then stops the exporters. Called once, at the very end of service_starter().
pub fn shutdown_telemetry() {
    let results = [
        ("traces", TRACER_PROVIDER.get().map(|p| p.shutdown())),
        ("metrics", METER_PROVIDER.get().map(|p| p.shutdown())),
    ];

    let mut flushed = false;
    let mut failed = false;
    for (what, result) in results {
        match result {
            Some(Ok(())) => flushed = true,
            // Never initialized, or a second call: the first one already sent everything.
            None | Some(Err(OTelSdkError::AlreadyShutdown)) => {}
            Some(Err(e)) => {
                failed = true;
                warn!("telemetry flush failed, the last {what} may be lost: {e}");
            }
        }
    }
    if flushed && !failed {
        info!("telemetry flushed");
    }
}
