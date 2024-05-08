use js_sys::{wasm_bindgen, Object};
use wasm_bindgen::JsValue;

use crate::wasm_bindgen::prelude::wasm_bindgen;

mod utils;

use crate::utils::{execute_http_request, HttpMethod, RequestBody};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    user_id: u32,
    id: u32,
    title: String,
    body: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn fetch_data() {
    wasm_bindgen_futures::spawn_local(async {
        let url = "https://jsonplaceholder.typicode.com/posts";
        let body = None;

        let response: serde_json::Value =
            match execute_http_request::<Post, ()>(HttpMethod::GET, url, body).await {
                Ok(data) => serde_json::to_value(data).unwrap(),
                Err(e) => {
                    log(&format!("Error fetching data: {:?}", e));
                    serde_json::Value::default() // Return a default value
                }
            };

        log(&format!("Response: {:?}", response));
    });
}

#[wasm_bindgen(start)]
pub fn run() {
    log("Hello from Rust!");
    fetch_data();
}
