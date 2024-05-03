use fastly::http::StatusCode;
use fastly::{Error,Response};
use uuid::Uuid;
use std::time::{Duration,SystemTime};

use crate::cache::{get_raw,write_raw};

struct Session {
	id: String,
	last_heartbeat: SystemTime,
}

pub fn write_sessions(sessions: Vec<String>) -> Result<(),Error> {
	write_raw("lbsessions", sessions)
}

pub fn add_session(id: &str) -> Result<Vec<String>,Error> {
	match get_raw::<String>("lbsessions") {
		Ok(mut s) => {
			s.push(id.to_string());
			Ok(s)
		},
		Err(e) => {
			println!("Error getting sessions: {:?}", e);
			Err(e)
		}
	}
}

pub fn new_session() -> String {
	Uuid::new_v4().to_string()
}


pub fn verify_session(id: &str) -> bool {
	match get_raw::<String>("lbsessions") {
		Ok(sessions) => {
			sessions.iter().find(|s| *s == id).is_some()
		},
		_ => false
	}
}
