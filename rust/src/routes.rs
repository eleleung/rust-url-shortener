extern crate hyper;
extern crate serde_json;
extern crate url;
extern crate regex;

use std::collections::HashMap;
use std::iter::FromIterator;
use std::error::Error;

use hyper::{Body, Response, Request, Method, Chunk};
use futures::{future, Future, Stream};
use regex::Regex;
use serde_json::{from_str, Value, to_string};
use url::{Url};

use super::server::{VromioServer, VromioApi};
use super::lib::{not_found, success, method_not_allowed, bad_request, redirect, not_authorized};

static URL_SHORTENER_KEY: &str = "VERYSECRETayylmaoKEYURLS6969";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item=Response<Body>, Error=GenericError> + Send>;

fn parse_uri_param<K, V>(host: &str, uri: &str) -> Result<HashMap<K,V>, Box<Error>>
    where HashMap<K, V>: FromIterator<(String, String)>
{
    let full_url = format!("{host}{uri}", host = host, uri = uri);

    let req_url = Url::parse(&full_url)?;
    let params: HashMap<K,V> = req_url.query_pairs().into_owned().collect();

    Ok(params)
}

fn parse_body(body: Chunk) -> Result<Vec<String>, Box<Error>> {
    let body_str = String::from_utf8(body.to_vec())?;
    let urls: Vec<String> = from_str(&body_str)?;

    Ok(urls)
}

pub fn svc_routes(req: Request<Body>, server: &VromioServer) -> ResponseFuture {
    lazy_static! {
        static ref analytics_route: Regex = Regex::new(r"/urls/analytics/.*").unwrap();
        static ref analytics_list_route: Regex = Regex::new(r"/urls/analytics").unwrap();
        static ref urls_route: Regex = Regex::new(r"/urls/.*").unwrap();
    }

    let path = req.uri().path();
    let method = req.method();

    if analytics_route.is_match(path) {
        match method {
            &Method::GET => {
                let path_suffix = req.uri().path().trim_start_matches("/urls/analytics/").to_string();

                let urls: Vec<String> = path_suffix
                    .split(",")
                    .map(|s| String::from(s))
                    .collect();

                let response = match server.analytics(urls) {
                    Ok(analytics) => {
                        success(to_string(&analytics).unwrap())
                    },
                    Err(_err) => {
                        not_found(Some("url analytics not found"))
                    }
                };
                return Box::new(future::ok(response));
            }
            _ => {
                return Box::new(future::ok(method_not_allowed("url not found")));
            }
        }
    }

    if analytics_list_route.is_match(path) {
        match method {
            &Method::POST => {
                let server_ref = server.clone();
                let response = req
                    .into_body()
                    .concat2()
                    .from_err()
                    .and_then(move |body| {
                        let urls = match parse_body(body) {
                            Ok(b) => b,
                            Err(_e) => vec![]
                        };

                        if urls.is_empty() {
                            let response = bad_request("Must send a list of strings");

                            Ok(response)
                        } else {
                            let result = match &server_ref.analytics(urls) {
                                Ok(analytics) => {
                                    success(to_string(&analytics).unwrap())
                                },
                                Err(_err) => {
                                    not_found(Some("url analytics not found"))
                                }
                            };

                            Ok(result)
                        }
                    });

                return Box::new(response);
            }
            _ => {
                return Box::new(future::ok(method_not_allowed("url not found")));
            }
        }
    }

    if urls_route.is_match(path) {
        match method {
            &Method::GET => {
                let path_suffix = req.uri().path().trim_start_matches("/urls/").to_string();

                let url: Option<String> = match server.fetch_url(req, &path_suffix.as_str()) {
                    Ok(url) => url,
                    Err(_e) => None
                };

                if url.is_some() {
                    return Box::new(future::ok(redirect(&url.unwrap())));
                } else {
                    return Box::new(future::ok(not_found(Some("url not found"))));
                }
            }
            &Method::POST => {
                match create_short_url(req, server) {
                    Ok(resp) => return resp,
                    Err(_err) => return Box::new(future::ok(not_found(Some("could not find analytics"))))
                };
            }
            _ => {
                return Box::new(future::ok(method_not_allowed("unsupported method")))
            }
        }
    }

    return Box::new(future::ok(not_found(Some("Not Found"))));
}

fn create_short_url(req: Request<Body>, server: &VromioServer) -> Result<ResponseFuture, Box<Error>> {
    let uri = req.uri().to_string();
    let host = req.headers().get("host").unwrap().to_str().unwrap().to_string();

    let params = parse_uri_param(&host, &uri)?;
    let sid = params.get("sid");

    if sid.is_some() {
        let path_suffix = req.uri().path().trim_start_matches("/urls/").to_string();
        let server_ref = server.clone();

        if path_suffix.is_empty() && sid.unwrap() == URL_SHORTENER_KEY {
            let response = req
                .into_body()
                .concat2()
                .from_err()
                .and_then(move |body| {
                    let str = String::from_utf8(body.to_vec())?;
                    let data: Value = from_str(&str)?;

                    let url = match &server_ref.shorten(&data["url"].as_str().unwrap()) {
                        Ok(u) => {
                            let fmt_url = format!("https://{host}/{result}", host = host, result = u);
                            let json = to_string(&fmt_url)?;

                            success(json)
                        },
                        Err(_e) => {
                            not_found(Some("could not shorten url"))
                        }
                    };

                    Ok(url)
                });

            return Ok(Box::new(response));
        } else {
            return Ok(Box::new(future::ok(not_found(Some("url not found")))));
        }
    } else {
        return Ok(Box::new(future::ok(not_authorized("missing sid"))));
    }
}