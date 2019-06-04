use base64::URL_SAFE_NO_PAD;
use chrono::prelude::Utc;
use hyper::{Body, Response, Version, StatusCode};
use rand::Rng;

pub fn new_id() -> String {
    let mut now = Utc::now().timestamp_millis();
    let mut bytes = rand::thread_rng().gen::<[u8; 12]>();

    for n in 0..6 {
        bytes[5 - n] = now as u8;
        now = now >> 8;
    }

    let id = base64::encode_config(&bytes, URL_SAFE_NO_PAD);

    return id;
}

pub fn redirect(url: &str) -> Response<Body> {
    Response::builder()
        .version(Version::HTTP_11)
        .status(StatusCode::SEE_OTHER)
        .header("location", url)
        .body(Body::empty())
        .unwrap()
}

pub fn not_authorized(body: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from(String::from(body)))
        .unwrap()
}

pub fn not_found(body: Option<&str>) -> Response<Body> {
    match body {
        Some(b) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(String::from(b)))
                .unwrap()
        },
        None => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()
        }
    }
}

pub fn method_not_allowed(body: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from(String::from(body)))
        .unwrap()
}

pub fn success<T>(body: T) -> Response<Body> where hyper::Body: std::convert::From<T> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))
        .unwrap()
}

pub fn bad_request(body_msg: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(String::from(body_msg)))
        .unwrap()
}