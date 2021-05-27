#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::env;

use log::{debug, error, log_enabled, info, Level};
use dotenv::dotenv;
use actix_web::{web, get, middleware, App, HttpResponse, HttpServer};
use listenfd::ListenFd;
use serde_json::json;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod route_static;
mod route_users;
mod errors;
mod models;
mod schema;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_web=info, debug");
    env_logger::init();

    let database_url = 
        env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let ip = 
        env::var("IP")
        .expect("IP not set")
        .parse::<Ipv4Addr>()
        .expect("Unable to parse IP");

    let port = 
        env::var("PORT")
        .expect("PORT not set")
        .parse::<u16>()
        .expect("Unable to parse PORT");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .configure(route_users::configure)
            .configure(route_static::configure)
    });

    // ListenFD to recompile with running server
    // systemfd --no-pid -s http::5000 -- cargo watch -x run
    let mut listenfd = ListenFd::from_env();

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind((ip, port))?,
    };

    server
        .run()
        .await?;

    Ok(())
}
