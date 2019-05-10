extern crate hyper;

use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

static TEXT: &str = "Hello, World!";

fn main() {
    let addr = ([0, 0, 0, 0], 6980).into();

    let new_svc = || {
        service_fn_ok(|_req|{
            Response::new(Body::from(TEXT))
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
