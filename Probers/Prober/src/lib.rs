use reqwest;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Url;
use reqwest::Method;
pub const URL: &str = "https://graphql.anilist.co";
use std::str::FromStr;
use std::error::Error;


pub async fn prober(client: &reqwest::Client, request: Request_Builder) -> serde_json::Value {
    query_executer(request.build(client).send()).await
}
/// should only validate that its valid
pub struct Request_Builder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    payload: Option<String>,
}
pub fn build_headermap(headers: Vec<(String, String)>) -> Result<HeaderMap, Box<dyn Error + Send + Sync>> {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        map.append(
            HeaderName::from_str(&name)?,
            HeaderValue::from_str(&value)?
        );
    }
    Ok(map)
}
impl Request_Builder {
    pub fn new(
        url: String,
        method: String,
        headers: Vec<(String, String)>,
        payload: Option<String>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {

        Ok(Request_Builder {
            url: Url::parse(&url)?,
            method: Method::from_str(&method)?,
            headers: build_headermap(headers)?,
            payload: payload,
        })
    }

    fn build(&self, client: &reqwest::Client) -> reqwest::RequestBuilder {
        let builder = client
            .request(self.method.clone(), self.url.clone())
            .headers(self.headers.clone());
        match &self.payload {
            Some(payload) => builder.body(payload.clone()),
            None => builder,
        }
    }
}

async fn query_executer<F>(query: F) -> serde_json::Value
where
F: Future<Output = Result<reqwest::Response, reqwest::Error>>,
{
    let start = Instant::now();
    match query.await {
        Ok(r) => {
            let end = Instant::now();
            let time = end.duration_since(start).as_secs();
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let ok = status.is_success() && !text.contains("\"errors\"");
            response_builder (
                time,
                ok,
                status.as_u16(),
                latency_ms
            )
        }
        Err(f) =>  {
            let end = Instant::now();
            let time = end.duration_since(start).as_secs();
            response_builder (
                time,
                false,
                f.status().map_or(0, |s| s.as_u16()),
                u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
            )
        }
    }
}
    pub fn response_builder(time: u64, ok: bool, status: u16, latency: u64) -> serde_json::Value {
        json!({
            "time": time,
            "ok": ok,
            "status": status,
        "latency": latency
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_headermap() {
        let map = build_headermap(vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ]).unwrap();
        assert_eq!(map["content-type"], "application/json");
    }

    #[test]
    fn rejects_bad_url() {
        assert!(Request_Builder::new("not a url".to_string(), "POST".to_string(), vec![], None).is_err());
    }
}
