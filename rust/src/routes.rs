extern crate hyper;
extern crate serde_json;
extern crate url;

use hyper::{Body, Response, Request, Method, StatusCode, Version};
use futures::{future, Future, Stream};
use serde_json::{from_str, Value, to_string};
use url::{Url};

use std::collections::HashMap;
use std::iter::FromIterator;

use super::server::{VromioServer, VromioApi};

static NOT_FOUND: &[u8] = b"Not Found";
static SUCCESS: &[u8] = b"Success";
static URL_SHORTENER_KEY: &str = "VERYSECRETayylmaoKEYURLS6969";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item=Response<Body>, Error=GenericError> + Send>;

fn parse_uri_param<K, V>(uri: &str) -> HashMap<K,V>
    where HashMap<K, V>: FromIterator<(String, String)>
{
    let req_url = Url::parse(&uri).unwrap();
    let params: HashMap<K,V> = req_url.query_pairs().into_owned().collect();

    params
}

fn redirect(url: &str) -> Response<Body> {
    Response::builder()
        .version(Version::HTTP_11)
        .status(StatusCode::SEE_OTHER)
        .header("location", url)
        .body(Body::empty())
        .unwrap()
}

pub fn svc_routes(req: Request<Body>, server: &VromioServer) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(SUCCESS))
                .unwrap()
            ))
        },
        (&Method::GET, "/urls/.*") => {
            let path_suffix = req.uri().path().trim_start_matches("/v2/urls").to_owned();

            let url: Option<String> = server.fetch_url(req, &path_suffix);

            if url.is_some() {
                Box::new(future::ok(redirect(&url.unwrap())))
            } else {
                Box::new(future::ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(String::from("Url not found")))
                    .unwrap()
                ))
            }
        },
        (&Method::POST, "/urls/.*") => {
            let uri = req.uri().to_string();
            let host = req.uri().host().unwrap().to_string();

            let params = parse_uri_param(&uri);
            let sid = params.get("sid");
            let path_suffix = req.uri().path().trim_start_matches("/v2/urls").to_string();
            let server_owned = server.clone();

            if path_suffix.is_empty() && sid.unwrap() == URL_SHORTENER_KEY {
                let response = req
                    .into_body()
                    .concat2()
                    .from_err()
                    .and_then(move |body| {
                        let str = String::from_utf8(body.to_vec()).unwrap();
                        let data : Value = from_str(&str).unwrap();

                        let url = &server_owned.shorten(&data.to_string());
                        let fmt_url = format!("https://{host}/{result}", host = host, result = url);
                        let json = to_string(&fmt_url).unwrap();
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(json))
                            .unwrap();

                        Ok(response)
                    });

                return Box::new(response);
            } else {
                Box::new(future::ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("url not found"))
                    .unwrap()
                ))
            }
        },
        (&Method::GET, "/urls/analytics") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(String::from("Url not found")))
                .unwrap()
            ))
        },
        (&Method::POST, "/urls/analytics") => {
            let server_owned = server.clone();
            let response = req
                .into_body()
                .concat2()
                .from_err()
                .and_then(move |body| {
                    let body_str = String::from_utf8(body.to_vec()).unwrap();
                    let urls: Vec<&str> = from_str(&body_str).unwrap();

                    if urls.is_empty() {
                        let response = Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from("Must send a list of strings"))
                            .unwrap();

                        Ok(response)
                    } else {
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(to_string(&server_owned.analytics(urls)).unwrap()))
                            .unwrap()
                        )
                    }
                });

            Box::new(response)
        },
        (&Method::GET, "/urls/analytics/*") => {
            let path_suffix = req.uri().path().trim_start_matches("/v2/urls/analytics/");

            let urls: Vec<&str> = path_suffix
                .split(",")
                .collect();

            let analytics = server.analytics(urls);

            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(to_string(&analytics).unwrap()))
                .unwrap()
            ))
        },
        (&Method::POST, "/urls/analytics/*") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(String::from("Url not found")))
                .unwrap()
            ))
        },
        _ => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(NOT_FOUND))
                .unwrap()
            ))
        }
    }
}