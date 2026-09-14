//! Real browser HTTP ceremonies and cookies, using the retained software passkey.
//! This is not a conforming-browser resident-key compatibility claim.
use crate::{account_enrollment_producer::EnrolledAccount, native_fixture::TestResult};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use url::Url;
use webauthn_authenticator_rs::prelude::RequestChallengeResponse;
#[derive(Clone)]
pub struct Browser {
    pub cookies: BTreeMap<String, String>,
    pub proof: String,
    pub public_origin: Url,
}
impl Browser {
    pub fn cookie(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
    pub fn absorb(&mut self, h: &reqwest::header::HeaderMap) -> TestResult {
        for value in h.get_all(reqwest::header::SET_COOKIE) {
            let pair = value.to_str()?.split(';').next().ok_or("cookie")?;
            let (k, v) = pair.split_once('=').ok_or("cookie pair")?;
            self.cookies.insert(k.into(), v.into());
        }
        Ok(())
    }
    pub async fn refresh(&mut self, client: &reqwest::Client, endpoint: &Url) -> TestResult {
        let proof = client
            .get(endpoint.join("/api/v2/auth/csrf")?)
            .header("X-Console-CSRF", "fetch")
            .header(
                reqwest::header::ORIGIN,
                self.public_origin.as_str().trim_end_matches('/'),
            )
            .header(reqwest::header::COOKIE, self.cookie())
            .send()
            .await?;
        assert_eq!(proof.status(), 200);
        self.proof = proof.json::<Value>().await?["csrf_proof"]
            .as_str()
            .ok_or("fresh proof")?
            .into();
        let response = self
            .request(
                client,
                endpoint,
                reqwest::Method::POST,
                "/api/v2/auth/token/refresh",
                Some(json!({})),
            )
            .await?;
        assert_eq!(response.status(), 200);
        self.absorb(response.headers())?;
        Ok(())
    }
    pub async fn request(
        &self,
        client: &reqwest::Client,
        origin: &Url,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> TestResult<reqwest::Response> {
        let mut q = client
            .request(method, origin.join(path)?)
            .header(
                reqwest::header::ORIGIN,
                self.public_origin.as_str().trim_end_matches('/'),
            )
            .header(reqwest::header::COOKIE, self.cookie());
        if !self.proof.is_empty() {
            q = q.header("X-Console-CSRF", &self.proof);
        }
        if let Some(body) = body {
            q = q.json(&body);
        }
        Ok(q.send().await?)
    }
}
pub async fn login(
    client: &reqwest::Client,
    origin: &Url,
    account: &mut EnrolledAccount,
) -> TestResult<Browser> {
    login_at(client, origin, origin, account).await
}
pub async fn login_at(
    client: &reqwest::Client,
    origin: &Url,
    public_origin: &Url,
    account: &mut EnrolledAccount,
) -> TestResult<Browser> {
    let mut browser = Browser {
        cookies: BTreeMap::new(),
        proof: String::new(),
        public_origin: public_origin.clone(),
    };
    let response = browser
        .request(
            client,
            origin,
            reqwest::Method::POST,
            "/api/v2/auth/passkey/login/start",
            Some(json!({})),
        )
        .await?;
    assert_eq!(response.status(), 200);
    browser.absorb(response.headers())?;
    let start: Value = response.json().await?;
    let mut options = start["public_key_options"].clone();
    options["publicKey"]["allowCredentials"]
        .as_array_mut()
        .ok_or("full challenge allow list")?
        .push(json!({"type":"public-key","id":account.credential_id}));
    let challenge: RequestChallengeResponse = serde_json::from_value(options)?;
    let assertion = account
        .authenticator
        .do_authentication(public_origin.clone(), challenge)?;
    let response = browser
        .request(
            client,
            origin,
            reqwest::Method::POST,
            "/api/v2/auth/passkey/login/finish",
            Some(json!({"ceremony_id":start["ceremony_id"],"assertion":assertion})),
        )
        .await?;
    assert_eq!(response.status(), 200);
    browser.absorb(response.headers())?;
    let established: Value = response.json().await?;
    assert_eq!(
        established["account"]["account_id"],
        json!(account.account_id)
    );
    assert!(
        browser
            .cookies
            .contains_key("__Host-console_account_session")
            && browser
                .cookies
                .contains_key("__Host-console_account_refresh")
    );
    let response = client
        .get(origin.join("/api/v2/auth/csrf")?)
        .header("X-Console-CSRF", "fetch")
        .header(
            reqwest::header::ORIGIN,
            public_origin.as_str().trim_end_matches('/'),
        )
        .header(reqwest::header::COOKIE, browser.cookie())
        .send()
        .await?;
    assert_eq!(response.status(), 200);
    browser.proof = response.json::<Value>().await?["csrf_proof"]
        .as_str()
        .ok_or("proof")?
        .to_string();
    Ok(browser)
}
