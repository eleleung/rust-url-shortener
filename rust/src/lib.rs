extern crate rand;

use chrono::prelude::Utc;

pub fn new_id() -> i64 {
    return Utc::now().timestamp_millis();
}