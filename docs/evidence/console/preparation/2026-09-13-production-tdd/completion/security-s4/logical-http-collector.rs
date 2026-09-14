//! Real HTTP body/header collector; no normalization, secret redaction, fake
//! response or response-length substitution. Clock/random coupling is caller duty.
use crate::{browser_auth_producer::Browser, native_fixture::TestResult};
use serde_json::{Value, json};
use url::Url;
pub async fn capture(
    client: &reqwest::Client,
    browser: &Browser,
    origin: &Url,
    path: &str,
) -> TestResult<Value> {
    let response = browser
        .request(client, origin, reqwest::Method::GET, path, None)
        .await?;
    let status = response.status().as_u16();
    let headers=response.headers().iter().map(|(k,v)|Ok(json!({"name":k.as_str(),"value_base64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,v.as_bytes())}))).collect::<TestResult<Vec<Value>>>()?;
    let body = response.bytes().await?;
    assert!(body.len() <= 8 * 1024 * 1024);
    Ok(
        json!({"path":path,"status":status,"headers":headers,"body_length":body.len(),"body_base64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,&body)}),
    )
}
