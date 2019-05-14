extern crate hyper;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use std::env;
use std::io;
use std::thread;

use hyper::{Body, Response, Server, Request, Method, StatusCode};
use hyper::service::service_fn;

use futures::{future, Future};

use r2d2_postgres::PostgresConnectionManager;

static NOT_FOUND: &[u8] = b"Not Found";
static SUCCESS: &[u8] = b"Success";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<Future<Item=Response<Body>, Error=GenericError> + Send>;

fn svc_routes(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(SUCCESS))
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

fn main() {
    let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_username = env::var("DB_USER").expect("DB_USER must be set");

    let db_manager = PostgresConnectionManager::new(
        format!("postgres://{db}:{pass}@{host}", db = db_username, pass = db_password, host = db_host),
        r2d2_postgres::TlsMode::None
    ).unwrap();

    let db_pool = r2d2::Pool::new(db_manager).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    }).unwrap();

    let pool = db_pool.clone();

    let handler = thread::spawn(move || {
        let client = pool.get().unwrap();
        client
            .execute("CREATE TABLE IF NOT EXISTS ShortUrl (
                id bigint check (id > 0) NOT NULL,
                code varchar(128) NOT NULL,
                url varchar(4096) NOT NULL,
                expiry bigint NOT NULL,
                PRIMARY KEY (id),
                CONSTRAINT code UNIQUE (code)
            )", &[])
            .unwrap();

        client
            .execute("CREATE TABLE IF NOT EXISTS ShortUrlClick (
                id bigint check (id > 0) NOT NULL,
                url bigint NOT NULL,
                time bigint NOT NULL,
                addr varchar(32) NOT NULL,
                ref varchar(4096) NOT NULL,
                agent varchar(4096) NOT NULL,
                PRIMARY KEY (id)
            )", &[])
            .unwrap();

        client
            .execute("CREATE INDEX urlTime ON ShortUrlClick (url, time)", &[])
            .unwrap();
    });

    handler.join().unwrap();

    let addr = ([0, 0, 0, 0], 6980).into();

    let new_svc = || {
        service_fn(|_req|{
            svc_routes(_req)
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
