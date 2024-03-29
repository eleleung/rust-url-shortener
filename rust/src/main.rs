#[macro_use] extern crate lazy_static;

use std::env;
use std::io;
use std::thread;

use hyper::{Server};
use hyper::service::service_fn;
use hyper::rt::run;

use futures::{Future, future};

use r2d2_postgres::PostgresConnectionManager;

use routes::svc_routes;
use server::VromioServer;

mod lib;
mod models;
mod random;
mod routes;
mod server;

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

    let client = pool.get().unwrap();

    client
        .execute("CREATE TABLE IF NOT EXISTS ShortUrl (
                id varchar(128) NOT NULL,
                code varchar(128) NOT NULL,
                url varchar(4096) NOT NULL,
                expiry timestamp NOT NULL,
                PRIMARY KEY (id),
                CONSTRAINT code UNIQUE (code)
            )", &[])
        .unwrap();

    client
        .execute("CREATE TABLE IF NOT EXISTS ShortUrlClick (
                id varchar(128) NOT NULL,
                url varchar(128) NOT NULL,
                time timestamp NOT NULL,
                addr varchar(32) NOT NULL,
                referer varchar(4096) NOT NULL,
                agent varchar(4096) NOT NULL,
                PRIMARY KEY (id)
            )", &[])
        .unwrap();

    client
        .execute("CREATE INDEX IF NOT EXISTS urlTime ON ShortUrlClick (url, time)",
                 &[])
        .unwrap();

    let addr = ([0, 0, 0, 0], 6980).into();

    run(future::lazy(move || {
        let new_svc = move || {
            let server_pool = db_pool.clone();

            let vromio = VromioServer {
                data_source: server_pool
            };

            service_fn(move |_req| {
                svc_routes(_req, &vromio)
            })
        };

        let server = Server::bind(&addr)
            .serve(new_svc)
            .map_err(|e| eprintln!("server error: {}", e));

        server
    }));
}
