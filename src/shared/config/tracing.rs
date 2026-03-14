use crate::shared::utils::constants::{OTLP_ENDPOINT, OTLP_SAMPLING_RATIO, OTLP_SERVICE_NAME};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_subscriber::{
    EnvFilter, Registry, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
    pub sampling_ratio: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: OTLP_ENDPOINT.clone(),
            service_name: OTLP_SERVICE_NAME.to_owned(),
            sampling_ratio: *OTLP_SAMPLING_RATIO,
        }
    }
}

impl TracingConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}

fn init_tracer_provider(config: &TracingConfig) -> Option<SdkTracerProvider> {
    let endpoint = config.otlp_endpoint.as_ref()?;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .ok()?;

    let resource = Resource::builder()
        .with_attribute(opentelemetry::KeyValue::new(
            SERVICE_NAME,
            config.service_name.clone(),
        ))
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::TraceIdRatioBased(config.sampling_ratio))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    Some(provider)
}

pub fn init_tracing(config: TracingConfig) -> TracingGuard {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info").add_directive("kanban_be=debug".parse().unwrap())
    });

    let tracer_provider = init_tracer_provider(&config);

    let registry = Registry::default().with(env_filter);

    let layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_span_events(FmtSpan::NONE);

    if let Some(ref provider) = tracer_provider {
        let tracer = provider.tracer(config.service_name.clone());
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        registry.with(layer).with(otel_layer).init();
    } else {
        registry.with(layer).init();
    }

    TracingGuard { tracer_provider }
}

pub struct TracingGuard {
    tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.tracer_provider.take()
            && let Err(err) = provider.shutdown()
        {
            eprintln!("Failed to shutdown tracer provider: {:?}", err);
        }
    }
}
