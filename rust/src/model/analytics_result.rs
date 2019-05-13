use std::hash::HashMap;

use link_clicks::LinkClicks;

pub struct AnalyticsResult {
    links: HashMap<String, LinkClicks>
}