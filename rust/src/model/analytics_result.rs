use std::collections::hash_map::{HashMap};

use super::link_clicks::LinkClicks;

pub struct AnalyticsResult {
    links: HashMap<String, LinkClicks>
}