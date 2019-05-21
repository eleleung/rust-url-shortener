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
        let id = new_id();
        let token = SecureRandomBase64::generate();
        let expiry = Utc::now().naive_utc() + Duration::days(365);

        let stmt = "INSERT INTO ShortUrl (id, code, url, expiry)  VALUES ($1 $2 $3 $4)";

        let client = self.data_source
            .get()
            .unwrap();

        client
            .execute(stmt, &[&id, &token, &url, &expiry])
            .unwrap();

        return token.to_string();
    }

    fn analytics(&self, urls: Vec<&str>) -> AnalyticsResult {
        let query = "SELECT T2.id t2f0, T2.code t2f1, T2.url t2f2, T2.expiry t2f3
            FROM ShortUrl T2
            INNER JOIN ShortUrlClick T1 ON (T2.id = T1.url)
            WHERE T2.code IN ($1)";

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
            let referrer: String = row.get("ref");
            let agent: String = row.get("agent");
            let time: NaiveDateTime = row.get("time");
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
        let query = "SELECT T2.id t2f0, T2.code t2f1, T2.url t2f2, T2.expiry t2f3
        FROM ShortUrl T2 WHERE (T2.code = $1)";

        let client = self.data_source
            .get()
            .unwrap();

        let rows = client.query(query, &[&code]).unwrap();
        let row = rows.get(0);
        let url: Option<String> = match row.get("url") {
            Some(url) => url,
            None => None
        };

        if url.is_some() {
            let id: Option<i64> = match row.get("id") {
                Some(url) => url,
                None => None
            };

            if id.is_some() {
                let fwd_for = req.headers().get("x-forwarded-for").unwrap().to_str().unwrap_or("");
                let referer = req.headers().get("referer").unwrap().to_str().unwrap_or("");
                let agent = req.headers().get("user-agent").unwrap().to_str().unwrap_or("");

                self.record_click(id.unwrap(), fwd_for, referer, agent);
            }
        }

        url
    }
}

impl VromioServer {
    fn record_click(&self, url_id: i64, fwd_for: &str, referer: &str, agent: &str) {
        let stmt = "INSERT INTO ShortUrlClick (id, url, time, addr, ref, agent)
            VALUES ($1 $2 $3 $4 $5)";

        let client = self.data_source
            .get()
            .unwrap();

        let id = new_id();
        let time = Utc::now();

        client
            .execute(stmt, &[&id, &url_id, &time, &fwd_for, &referer, &agent])
            .unwrap();
    }
}