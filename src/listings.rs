use fastly::http::StatusCode;
use fastly::{Error,Response};
use uuid::Uuid;
use std::time::{Duration,SystemTime};
use serde::{Deserialize,Serialize};
use serde::de::DeserializeOwned;

use crate::valkeyrie::{get_raw,write_raw};

#[derive(Serialize,Deserialize,Debug,PartialOrd,PartialEq,Clone)]
pub struct ListingEntry {
	id: String,
	name: String,
	score: i32,
	rank: usize,
//	last_heartbeat: SystemTime,
}

impl ListingEntry {
	pub fn new(id: String, name: String, score: i32) -> Self {
		ListingEntry{
			id:id,
			name:name,
			score:score,
			rank: 0,
		}
	}
}

pub fn write_listings(listings: Vec<ListingEntry>) -> Result<(),Error> {
	write_raw("lblistings", listings)
}

pub fn add_or_update_listing(id: &str, name: &str, score: i32) -> Result<Vec<ListingEntry>,Error> {
	match get_raw::<ListingEntry>("lblistings") {
		Ok(mut listings) => {
			match listings.iter_mut().find(|l| l.id == id) {
				Some(mut l) => {
					l.score = score;
				},
				_ => {
					let listing = ListingEntry::new(id.to_string(), name.to_string() ,score);
					listings.push(listing);
				}
			}
			listings.sort_by(|l1,l2| l2.score.cmp(&l1.score));
			for i in 0..listings.len() {
				listings[i].rank = i + 1;
			}
			Ok(listings)
		},
		Err(e) => {
			println!("Error getting listings: {:?}", e);
			Err(e)
		}
	}
}

pub fn get_index_for_score(score: i32) -> Option<(usize, usize)> {
	match get_raw::<ListingEntry>("lblistings") {
		Ok(mut listings) => {
			for (index,listing) in listings.iter().enumerate() {
				if listing.score < score {
					return Some((index, listings.len()));
				}
			}
			return Some((match listings.len() {
				0 => 0,
				_ => listings.len()-1,
			},listings.len()));
		},
		_ => return None,
	}
}

pub fn get_listings(s: usize, num: usize) -> Result<Vec<ListingEntry>,Error> {
	match get_raw::<ListingEntry>("lblistings") {
		Ok(mut listings) => {
			if num == 0 {
				return Ok(listings);
			}
			let start = std::cmp::min(s,listings.len());
			let end = std::cmp::min(start+num, listings.len());
			Ok(listings[start..end].to_vec())
		},
		Err(e) => {
			Err(e)
		}
	}

	// let mut listings = vec![];
	// listings.push(ListingEntry{id: "0".to_string(), name: "Jessica".to_string(), score: 200});
	// listings.push(ListingEntry{id: "0".to_string(), name: "Esme".to_string(), score: 100});
	// listings.push(ListingEntry{id: "0".to_string(), name: "Zayden".to_string(), score: 57});

//	Ok(listings)
}