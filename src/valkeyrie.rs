use fastly::http::{Method};
use fastly::{Error, Request};
use serde::de::{DeserializeOwned};
use serde::Serialize;

const KV_GLOBAL: &str = "kvglobal";

pub fn get_raw<T: DeserializeOwned+std::fmt::Debug>(endpoint: &str) -> Result<Vec<T>, Error> {
	let kvreq = Request::get(format!("http://kv-global.vranish.dev/{}", endpoint))
	.with_method(Method::GET)
	.with_body_text_plain("");
	let resp = kvreq.send(KV_GLOBAL)?;
	let body_str = resp.into_body().into_string();

	if body_str == "Not Found" {
		return Ok(Vec::new());
	}

	match serde_json::from_str(&body_str) {
		Ok(v) => {
			let res : Vec<T> = v;
			Ok(res)
		},
		Err(e) => {
			Err(Error::msg(format!("Couldn't convert {} to json: {:?}", body_str, e)))
		}
	}
}

pub fn write_raw<T: Serialize>(endpoint: &str, data: Vec<T>) -> Result<(), Error> {
	let json_result = serde_json::to_string(&data);
	match json_result {
		Ok(json) => {
			let kvreq = Request::get(format!("http://kv-global.vranish.dev/{}", endpoint))
			.with_method(Method::POST)
			.with_body_text_plain(&json);
			let resp = kvreq.send(KV_GLOBAL);
			Ok(())
		},
		Err(e) => {
			Err(Error::msg(format!("Couldn't serialize to json: {}", e)))
		}
	}
}