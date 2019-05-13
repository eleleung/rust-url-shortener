use std::time::{Instant};

use id::*;

struct ShortUrlClick {
    id: Id,
    time: Instant,
    addr: String,
    referrer: String,
    agent: String
}