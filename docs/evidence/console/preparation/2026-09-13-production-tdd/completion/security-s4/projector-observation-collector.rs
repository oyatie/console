//! Concrete collector over the actual production serializer/renderer/cache/live/
//! event/egress encoders. This composes at their typed input boundary; HTTP socket
//! parity is the separate logical-http-collector and adapter enrollment gate.
use crate::native_fixture::TestResult;
use console_ontology_application::projection::{AuthorizedProjection, PublicObservationInputs};
use serde_json::{Value, json};
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub http: Value,
    pub ssr: Value,
    pub cache: Value,
    pub live: Vec<Value>,
    pub events: Vec<Value>,
    pub egress: Vec<u8>,
}
async fn response(r: axum::response::Response) -> TestResult<Value> {
    use base64::Engine;
    let status = r.status().as_u16();
    let mut headers = Vec::new();
    for name in r.headers().keys() {
        for v in r.headers().get_all(name) {
            headers.push(json!({"name":name.as_str(),"value_base64":base64::engine::general_purpose::STANDARD.encode(v.as_bytes())}));
        }
    }
    let bytes = axum::body::to_bytes(r.into_body(), 16 * 1024 * 1024).await?;
    Ok(
        json!({"status":status,"headers":headers,"body_length":bytes.len(),"body_base64":base64::engine::general_purpose::STANDARD.encode(bytes)}),
    )
}
pub async fn observe(
    p: &AuthorizedProjection,
    env: &PublicObservationInputs,
) -> TestResult<Observation> {
    // Each method below is an ordinary production boundary, required to be the one
    // called by its registered adapter. No test builds a projected output itself.
    let http =
        response(console_ontology_rest::projection::render_projection_response(p, env)?).await?;
    let ssr = response(console_app::projection::render_projection_ssr_response(
        p, env,
    )?)
    .await?;
    let entry = console_app::projection_cache::build_projection_cache_entry(p, env)?;
    let cache = json!({"key":entry.key_bytes(),"value":entry.value_bytes(),"ttl":entry.ttl_seconds(),"vary":entry.vary_inputs()});
    let mut live = Vec::new();
    for (ordinal, frame) in console_platform_realtime::projection::build_projection_frames(p, env)?
        .into_iter()
        .enumerate()
    {
        live.push(json!({"ordinal":ordinal,"opcode":frame.opcode(),"bytes":frame.encode()?}));
    }
    let mut events = Vec::new();
    for (ordinal, event) in
        console_platform_observability::safe_event::schedule_projection_events(p, env)?
            .into_iter()
            .enumerate()
    {
        events.push(json!({"ordinal":ordinal,"bytes":console_platform_observability::safe_event::encode_safe_operational_event(&event)?}));
    }
    let egress = console_ontology_rest::projection::encode_bound_object_projection(p, env)?;
    Ok(Observation {
        http,
        ssr,
        cache,
        live,
        events,
        egress,
    })
}
pub fn equal(a: &Observation, b: &Observation) -> bool {
    a == b
}
