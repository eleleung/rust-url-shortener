extern crate chrono;
extern crate hyper;
extern crate rand;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate time;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Vacant, Occupied};
use std::error::Error;

use chrono::prelude::*;
use chrono::NaiveDateTime;
use hyper::{Request, Body};
use r2d2_postgres::{PostgresConnectionManager};
use r2d2::Pool;
use time::{Duration};

use super::models::{AnalyticsResult, LinkClicks, ShortUrlClick};
use super::random::SecureRandomBase64;
use super::lib::{new_id};

#[derive(Clone)]
pub struct VromioServer {
    pub data_source: Pool<PostgresConnectionManager>
}

pub trait VromioApi {
    fn shorten(&self, url: &str) -> Result<String, Box<Error>>;
    fn analytics(&self, urls: Vec<&str>) -> Result<AnalyticsResult, Box<Error>>;
    fn fetch_url(&self, req: Request<Body>, code: &str) -> Result<Option<String>, Box<Error>>;
}

impl VromioApi for VromioServer {
    fn shorten(&self, url: &str) -> Result<String, Box<Error>> {
        let id = new_id();
        let token = SecureRandomBase64::generate();
        let expiry = Utc::now().naive_utc() + Duration::days(365);

        let stmt = "INSERT INTO ShortUrl (id, code, url, expiry)  VALUES ($1, $2, $3, $4)";

        let client = self.data_source.get()?;

        client.execute(stmt, &[&id, &token, &url, &expiry])?;

        return Ok(token.to_string());
    }

    fn analytics(&self, urls: Vec<&str>) -> Result<AnalyticsResult, Box<Error>> {
        let query = "SELECT T1.code code, T2.id id, T2.addr addr, T2.referer referer, T2.agent agent, T2.time click_time
            FROM ShortUrl T1
            INNER JOIN ShortUrlClick T2 ON (T2.url = T1.id)
            WHERE T1.code = ANY($1)";

        let client = self.data_source.get()?;

        let rows = client.query(query, &[&urls])?;
        let mut links: HashMap<String, LinkClicks>  = HashMap::new();

        for row in &rows {
            let code: String = row.get("code");

            let addr: String = row.get("addr");
            let referer: String = row.get("referer");
            let agent: String = row.get("agent");
            let time: NaiveDateTime = row.get("click_time");

            let link_click = ShortUrlClick {
                time,
                addr,
                referer,
                agent
            };

            match links.entry(code) {
                Vacant(e) => {
                    let link_clicks = LinkClicks {
                        clicks: vec![link_click]
                    };

                    e.insert(link_clicks);
                },
                Occupied(mut e) => {
                    e.get_mut().clicks.push(link_click);
                }
            }
        }

        Ok(AnalyticsResult {
            links
        })
    }

    fn fetch_url(&self, req: Request<Body>, code: &str) -> Result<Option<String>, Box<Error>> {
        let query = "SELECT id, code, url, expiry
        FROM ShortUrl WHERE (code = $1)";

        let client = self.data_source.get()?;

        let rows = client.query(query, &[&code])?;
        if rows.is_empty() {
            return Ok(None);
        }

        let row = rows.get(0);

        let id: Option<String> = row.get("id");
        let url: Option<String> = row.get("url");

        if url.is_some() && id.is_some() {
            let fwd_for = match req.headers().get("x-forwarded-for") {
                Some(fwd) => fwd.to_str().unwrap(),
                None => ""
            };

            let referer = match req.headers().get("referer") {
                Some(referer) => referer.to_str().unwrap(),
                None => ""
            };

            let agent = match req.headers().get("user-agent") {
                Some(agent) => agent.to_str().unwrap(),
                None => ""
            };

            self.record_click(&id.unwrap(), fwd_for, referer, agent)?;

            return Ok(url);
        } else {
            return Ok(None);
        }
    }
}

impl VromioServer {
    fn record_click(&self, url_id: &str, fwd_for: &str, referer: &str, agent: &str) -> Result<u64, Box<Error>> {
        let stmt = "INSERT INTO ShortUrlClick (id, url, time, addr, ref, agent)
            VALUES ($1, $2, $3, $4, $5, $6)";

        let client = self.data_source.get()?;

        let id = new_id();
        let time = Utc::now().naive_utc();

        Ok(client.execute(stmt, &[&id, &url_id, &time, &fwd_for, &referer, &agent])?)
    }
}