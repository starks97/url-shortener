use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::log;

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    DELETE,
    PUT,
}

impl HttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::DELETE => "DELETE",
            HttpMethod::GET => "GET",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
        }
    }
}

pub struct RequestBody<T> {
    data: T,
}

impl<T> RequestBody<T> {
    pub fn new(data: T) -> Self {
        RequestBody { data }
    }
}

pub(super) async fn execute_http_request<R, P>(
    method: HttpMethod,
    url: &str,
    body: Option<&RequestBody<P>>,
) -> Result<R, JsValue>
where
    R: for<'de> serde::Deserialize<'de>,
    P: serde::Serialize,
{
    let mut opts = RequestInit::new();
    opts.mode(RequestMode::Cors);
    match method {
        HttpMethod::GET => {
            opts.method("GET");
        }

        _ => {
            opts.method(method.as_str());
            if let Some(b) = body {
                // Only set the body if the method is not GET
                opts.body(Some(&wasm_bindgen::JsValue::from_str(
                    &serde_json::to_string(&b.data).unwrap(),
                )));
            }
        }
    }

    let request = match Request::new_with_str_and_init(url, &opts) {
        Ok(r) => Some(r),
        Err(e) => {
            log(&format!("Error creating request: {:?}", e));
            None
        }
    };

    let _ = request
        .as_ref()
        .unwrap()
        .headers()
        .set("Content-Type", "application/json");

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request.as_ref().unwrap())).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await.unwrap();

    let data: R = serde_json::from_str(&json.as_string().unwrap()).unwrap();

    Ok(data)
}
