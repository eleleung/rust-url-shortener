use std::collections::hash_map::{HashMap};
use std::time::{Instant};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Id {
    pub value: i64
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ShortUrlClick {
    pub id: Id,
    pub time: Instant,
    pub addr: String,
    pub referrer: String,
    pub agent: String
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct LinkClicks {
    pub clicks: Vec<ShortUrlClick>
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct AnalyticsResult {
    pub links: HashMap<String, LinkClicks>
}