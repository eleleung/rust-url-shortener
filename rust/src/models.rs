use std::collections::hash_map::{HashMap};
use std::time::{Instant};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Id {
    value: i64
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ShortUrlClick {
    id: Id,
    time: Instant,
    addr: String,
    referrer: String,
    agent: String
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct LinkClicks {
    clicks: Vec<ShortUrlClick>
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct AnalyticsResult {
    links: HashMap<String, LinkClicks>
}