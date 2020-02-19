use std::collections::hash_map::{ HashMap };
use std::collections::BTreeMap;

use error::Result;
use jobs;

// ----------------------------------------------------------------------------
// Interface

#[derive(Serialize)]
struct Response<'a> {
    status_code: u16,
    headers: HashMap<&'a str, &'a str>,
    body: &'a str,
}

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure.
byond_fn! { http_request_blocking(method, url, body, headers) {
    let req = match construct_request(method, url, body, headers) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    match submit_request(req) {
        Ok(r) => Some(r),
        Err(e) => Some(e.to_string())
    }
} }

// Returns new job-id.
byond_fn! { http_request_async(method, url, body, headers) {
    let req = match construct_request(method, url, body, headers) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    Some(jobs::start(move || {
        match submit_request(req) {
            Ok(r) => r,
            Err(e) => e.to_string()
        }
    }))
} }

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure or WIP.
byond_fn! { http_check_request(id) {
    Some(jobs::check(id))
} }

// ----------------------------------------------------------------------------
// Shared HTTP client state

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

fn setup_http_client() -> reqwest::Client {
    use reqwest::{ Client, header::{ HeaderMap, USER_AGENT } };

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, format!("{}/{}", PKG_NAME, VERSION).parse().unwrap());

    Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = setup_http_client();
}

// ----------------------------------------------------------------------------
// Request construction and execution

fn construct_request(method: &str, url: &str, body: &str, headers: &str) -> Result<reqwest::RequestBuilder> {
    let mut req = match method {
        "post" => HTTP_CLIENT.post(url),
        "put" => HTTP_CLIENT.put(url),
        "patch" => HTTP_CLIENT.patch(url),
        "delete" => HTTP_CLIENT.delete(url),
        "head" => HTTP_CLIENT.head(url),
        _ => HTTP_CLIENT.get(url),
    };

    if !body.is_empty() {
        req = req.body(body.to_owned());
    }

    if !headers.is_empty() {
        let headers: BTreeMap<&str, &str> = serde_json::from_str(headers)?;
        for (key, value) in headers {
            req = req.header(key, value);
        }
    }

    Ok(req)
}

fn submit_request(req: reqwest::RequestBuilder) -> Result<String> {
    let mut response = req.send()?;

    let body = response.text()?;

    let mut resp = Response {
        status_code: response.status().as_u16(),
        headers: HashMap::new(),
        body: &body,
    };

    for (key, value) in response.headers().iter() {
        if let Ok(value) = value.to_str() {
            resp.headers.insert(key.as_str(), value);
        }
    }

    Ok(serde_json::to_string(&resp)?)
}
