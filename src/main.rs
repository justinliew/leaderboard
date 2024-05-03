//! Default Compute@Edge template program.

use fastly::http::{header, Method, StatusCode};
use fastly::{Error, Request, Response};
use fastly::http::header::HeaderValue;
use listings::{ListingEntry, add_or_update_listing, get_index_for_score, get_listings, write_listings};
use num;

mod listings;
mod sessions;
mod cache;

use crate::cache::get_raw;
use crate::sessions::{add_session,new_session,verify_session,write_sessions};

fn header_val(header: Option<&HeaderValue>) -> &str {
	match header {
		Some(h) => h.to_str().unwrap_or(""),
		None => "",
	}
}

/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.
#[fastly::main]
fn main(mut req: Request) -> Result<Response, Error> {
    // Make any desired changes to the client request.
//	req.set_header("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

	println!("Got {} {}", req.get_method(), req.get_path());
	if req.get_method() == Method::OPTIONS {
        return Ok(Response::from_status(StatusCode::OK)
			.with_header("Access-Control-Allow-Origin","*")
			.with_header("Access-Control-Allow-Headers","*")
			.with_header("Vary","Origin")
            .with_body_text_plain(""))
	}

    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests.
        &Method::GET | &Method::HEAD | &Method::POST => (),

        // Deny anything else.
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n"))
        }
    };

	// might need this
	// .with_header("Access-Control-Allow-Origin","*")
	// .with_header("Access-Control-Allow-Headers","*")
	// .with_header("Vary","Origin")

    // Pattern match on the path.
    match req.get_path() {
        // If request is to the `/` path, send a default response.
        "/" => Ok(Response::from_status(StatusCode::OK)
			.with_body_text_plain("Welcome to Leaderboard Service")),

		"/new_session" => {
			let id = new_session();
				match add_session(&id) {
				Ok(r) =>
					match write_sessions(r) {
						Ok(_) =>
							Ok(Response::from_status(StatusCode::OK)
							.with_header("Access-Control-Allow-Origin", "*")
							.with_body(id)),
						Err(e) =>
							Ok(Response::from_status(StatusCode::SERVICE_UNAVAILABLE))
					},
				Err(_) =>
					Ok(Response::from_status(StatusCode::SERVICE_UNAVAILABLE))
			}
		},

		// verify session with session list
		// look for session in leaderboard
		// insert or update,sort
		"/update_listing" => {
			let id = header_val(req.get_header("id"));
			let name = header_val(req.get_header("name"));
			if let Ok(score) = header_val(req.get_header("score")).parse::<i32>() {
				if !verify_session(id) {
					return Ok(Response::from_status(StatusCode::NOT_FOUND).with_header("Access-Control-Allow-Origin", "*"))
				}

				match add_or_update_listing(id,name,score) {
					Ok(listings) => {
						write_listings(listings);
						Ok(Response::from_status(StatusCode::OK)
						.with_header("Access-Control-Allow-Origin", "*")
					)
					},
					Err(e) => {
						Ok(Response::from_status(StatusCode::SERVICE_UNAVAILABLE))
					}
				}
			} else {
				Ok(Response::from_status(StatusCode::NOT_FOUND))
			}
		},

		// optional - find session
		// mark as "done"
		// TODO
		// "/finalize_listing" => {
		// 	Ok(Response::from_status(StatusCode::OK)
		// 	.with_body_text_plain("0"))
		// },

		// TODO - should we paginate the leaderboard?
		// get leaderboard
		"/around_me" => {
			if let Ok(score) = header_val(req.get_header("score")).parse::<i32>() {
				if let Some((current,max)) = get_index_for_score(score) {
					let current : usize = num::clamp(current as i32 - 5,0 as i32,max as i32) as usize;
					if let Ok(listings) = get_listings(current,10) {
						if let Ok(json) = serde_json::to_string(&listings) {
							return Ok(Response::from_status(StatusCode::OK)
							.with_header("Access-Control-Allow-Origin", "*")
							.with_body_text_plain(&json));
						}
					}
				}
			}
			Ok(Response::from_status(StatusCode::NOT_FOUND)
			.with_header("Access-Control-Allow-Origin", "*"))

		},

		// get leaderboard
		"/topten" => {
			if let Ok(listings) = get_listings(0,10) {
				if let Ok(json) = serde_json::to_string(&listings) {
					return Ok(Response::from_status(StatusCode::OK)
					.with_header("Access-Control-Allow-Origin", "*")
					.with_body_text_plain(&json));
				}
			}
			Ok(Response::from_status(StatusCode::OK)
			.with_header("Access-Control-Allow-Origin", "*")
			.with_body_text_plain("{}"))
		},
		// get leaderboard
		"/global" => {
			if let Ok(listings) = get_listings(1,0) {
				if let Ok(json) = serde_json::to_string(&listings) {
					return Ok(Response::from_status(StatusCode::OK)
					.with_header("Access-Control-Allow-Origin", "*")
					.with_body_text_plain(&json));
				}
			}
			Ok(Response::from_status(StatusCode::OK)
			.with_header("Access-Control-Allow-Origin", "*")
			.with_body_text_plain("{}"))
		},
        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
