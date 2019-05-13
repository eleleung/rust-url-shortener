use std::time::{Instant};

use super::id::*;

pub struct ShortUrlClick {
    id: Id,
    time: Instant,
    addr: String,
    referrer: String,
    agent: String
}