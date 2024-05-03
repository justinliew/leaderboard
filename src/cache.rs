use fastly::http::{Method};
use fastly::{Error, Request};
use serde::de::{DeserializeOwned};
use serde::Serialize;
use fastly::cache::core::{CacheKey, lookup, insert};
use std::io::Write;
use std::time::Duration;

const KV_GLOBAL: &str = "kvglobal";

pub fn get_raw<T: DeserializeOwned+std::fmt::Debug>(endpoint: &str) -> Result<Vec<T>, Error> {

	if let Some(entry) = lookup(CacheKey::copy_from_slice(endpoint.as_bytes())).execute().unwrap() {
		let body = entry.to_stream();
		if let Ok(b) = body {
			let body_str = b.into_string();
			if let Ok(json) = serde_json::from_str(&body_str) {
				return Ok(json);
			} else {
				return Err(Error::msg(format!("Couldn't convert {} to json", body_str)));
			}
		} else {
			return Err(Error::msg("Couldn't get valid body from cache"));
		}
	}
	return Ok(Vec::new())
}

pub fn write_raw<T: Serialize>(endpoint: &str, data: Vec<T>) -> Result<(), Error> {

	let json_result = serde_json::to_string(&data);
	match json_result {
		Ok(json) => {
			let mut writer = insert(CacheKey::copy_from_slice(endpoint.as_bytes()), Duration::from_secs(60*60*72)).execute().unwrap();
			writer.write_all(json.as_bytes()).unwrap();
			writer.finish().unwrap();
			Ok(())
		},
		Err(e) => {
			Err(Error::msg(format!("Couldn't serialize to json: {}", e)))
		}
	}
}