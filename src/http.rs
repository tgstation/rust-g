use crate::{error::Result, jobs};
use std::collections::{BTreeMap, HashMap};

// ----------------------------------------------------------------------------
// Interface

#[derive(Deserialize)]
struct RequestOptions {
    output_filename: Option<String>,
    body_filename: Option<String>,
}

#[derive(Serialize)]
struct Response<'a> {
    status_code: u16,
    headers: HashMap<&'a str, &'a str>,
    body: Option<&'a str>,
}

// If the response can be deserialized -> success.
// If the response can't be deserialized -> failure.
byond_fn! { http_request_blocking(method, url, body, headers, ...rest) {
    let req = match construct_request(method, url, body, headers, rest.first().map(|x| &**x)) {
        Ok(r) => r,
        Err(e) => return Some(e.to_string())
    };

    match submit_request(req) {
        Ok(r) => Some(r),
        Err(e) => Some(e.to_string())
    }
} }

// Returns new job-id.
byond_fn! { http_request_async(method, url, body, headers, ...rest) {
    let req = match construct_request(method, url, body, headers, rest.first().map(|x| &**x)) {
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
    use reqwest::{
        header::{HeaderMap, USER_AGENT},
        Client,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        format!("{}/{}", PKG_NAME, VERSION).parse().unwrap(),
    );

    Client::builder().default_headers(headers).build().unwrap()
}

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = setup_http_client();
}

// ----------------------------------------------------------------------------
// Request construction and execution

struct RequestPrep {
    req: reqwest::RequestBuilder,
    output_filename: Option<String>,
}

fn construct_request(
    method: &str,
    url: &str,
    body: &str,
    headers: &str,
    options: Option<&str>,
) -> Result<RequestPrep> {
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

    let mut output_filename = None;
    if let Some(options) = options {
        let options: RequestOptions = serde_json::from_str(options)?;
        output_filename = options.output_filename;
        if let Some(fname) = options.body_filename {
            req = req.body(std::fs::File::open(fname)?);
        }
    }

    Ok(RequestPrep {
        req,
        output_filename,
    })
}

fn submit_request(prep: RequestPrep) -> Result<String> {
    let mut response = prep.req.send()?;

    let body;
    let mut resp = Response {
        status_code: response.status().as_u16(),
        headers: HashMap::new(),
        body: None,
    };

    if let Some(output_filename) = prep.output_filename {
        std::io::copy(&mut response, &mut std::fs::File::create(&output_filename)?)?;
    } else {
        body = response.text()?;
        resp.body = Some(&body);
    }

    for (key, value) in response.headers().iter() {
        if let Ok(value) = value.to_str() {
            resp.headers.insert(key.as_str(), value);
        }
    }

    Ok(serde_json::to_string(&resp)?)
}
