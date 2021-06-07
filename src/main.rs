#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::time::Duration;
use std::env;

use log::{debug, error, log_enabled, info, Level};
use dotenv::dotenv;
use listenfd::ListenFd;
use serde_json::json;
use tokio;

use actix_web::{web, get, middleware, App, HttpResponse, HttpServer};
use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use device_tracking::Device2Track;

mod route_static;
mod route_users;
mod route_devices;
mod errors;
mod models;
mod schema;
mod device_tracking;
mod network;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_web=info, debug");
    env_logger::init();

    // let mut device1 = Device2Track::new("F6:FC:B4:5E:C8:2F");
    // let mut device1 = Device2Track::new("ee:f1:3b:99:db:7f");
    // tokio::spawn(async move { device1.begin() });

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
            .service(fs::Files::new("/js", "./templates/js"))
            .service(fs::Files::new("/css", "./templates/css"))
            .service(fs::Files::new("/assets", "./templates/assets"))
            .configure(route_users::configure)
            .configure(route_static::configure)
            .configure(route_devices::configure)
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
