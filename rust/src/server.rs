extern crate chrono;
extern crate hyper;
extern crate rand;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate time;
extern crate uuid;

use std::collections::HashMap;

use chrono::prelude::*;
use chrono::NaiveDateTime;
use hyper::{Request, Body};
use r2d2_postgres::{PostgresConnectionManager};
use r2d2::Pool;
use time::{Duration};

use super::models::{AnalyticsResult, LinkClicks, ShortUrlClick, Id};
use super::random::SecureRandomBase64;
use super::lib::{new_id};

#[derive(Clone)]
pub struct VromioServer {
    pub data_source: Pool<PostgresConnectionManager>
}

pub trait VromioApi {
    fn shorten(&self, url: &str) -> String;
    fn analytics(&self, urls: Vec<&str>) -> AnalyticsResult;
    fn fetch_url(&self, req: Request<Body>, code: &str) -> Option<String>;
}

impl VromioApi for VromioServer {
    fn shorten(&self, url: &str) -> String {
        let id: i64 = new_id();
        let token = SecureRandomBase64::generate();
        let expiry = Utc::now().naive_utc() + Duration::days(365);

        let stmt = "INSERT INTO ShortUrl (id, code, url, expiry)  VALUES ($1, $2, $3, $4)";

        let client = self.data_source
            .get()
            .unwrap();

        client
            .execute(stmt, &[&id, &token, &url, &expiry])
            .unwrap();

        return token.to_string();
    }

    fn analytics(&self, urls: Vec<&str>) -> AnalyticsResult {
        let query = "SELECT T1.code code, T2.id id, T2.addr addr, T2.ref referer, T2.agent agent, T2.time click_time
            FROM ShortUrl T1
            INNER JOIN ShortUrlClick T2 ON (T2.url = T1.id)
            WHERE T1.code = ANY($1)";

        let client = self.data_source
            .get()
            .unwrap();

        let rows = client.query(query, &[&urls]).unwrap();
        let mut links: HashMap<String, LinkClicks>  = HashMap::with_capacity(urls.len());

        // refactor entire thing
        for row in &rows {
            let code: String = row.get("code");

            let id_val: i64 = row.get("id");
            let addr: String = row.get("addr");
            let referrer: String = row.get("referer");
            let agent: String = row.get("agent");
            let time: NaiveDateTime = row.get("click_time");
            let id = Id {
                value: id_val
            };

            let link_click = ShortUrlClick {
                id,
                time,
                addr,
                referrer,
                agent
            };

            if links.contains_key(&code) {
                let mut link_clicks: LinkClicks = links.remove(&code).unwrap();
                link_clicks.clicks.push(link_click);

                links.insert(code.to_string(), link_clicks);
            } else {
                let link_clicks = LinkClicks {
                    clicks: vec![link_click]
                };

                links.insert(code.to_string(), link_clicks);
            }
        }

        AnalyticsResult {
            links
        }
    }

    fn fetch_url(&self, req: Request<Body>, code: &str) -> Option<String> {
        let query = "SELECT id, code, url, expiry
        FROM ShortUrl WHERE (code = $1)";

        let client = self.data_source
            .get()
            .unwrap();

        let rows = client.query(query, &[&code]).unwrap();
        if rows.is_empty() {
            return None;
        }

        let row = rows.get(0);

        let id: Option<i64> = row.get("id");
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

            self.record_click(id.unwrap(), fwd_for, referer, agent);

            return url;
        } else {
            return None;
        }
    }
}

impl VromioServer {
    fn record_click(&self, url_id: i64, fwd_for: &str, referer: &str, agent: &str) {
        let stmt = "INSERT INTO ShortUrlClick (id, url, time, addr, ref, agent)
            VALUES ($1, $2, $3, $4, $5, $6)";

        let client = self.data_source
            .get()
            .unwrap();

        let id = new_id();
        let time = Utc::now().naive_utc();

        client
            .execute(stmt, &[&id, &url_id, &time, &fwd_for, &referer, &agent])
            .unwrap();
    }
}