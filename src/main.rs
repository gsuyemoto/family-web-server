#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::time::Duration;
use std::env;
use std::cell::RefCell;

use log::{debug, error, log_enabled, info, Level};
use dotenv::dotenv;
use serde_json::json;

use tokio::task::JoinHandle;
use tokio;

use actix_web::{web, get, middleware, App, HttpResponse, HttpServer};
use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::device_tracking::DeviceTracking;

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

    let mut device_tracking = DeviceTracking::new(pool.clone());
    tokio::spawn(async move { device_tracking.begin() });

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(fs::Files::new("/js", "./templates/js"))
            .service(fs::Files::new("/css", "./templates/css"))
            .service(fs::Files::new("/assets", "./templates/assets"))
            .configure(route_users::configure)
            .configure(route_static::configure)
            .configure(route_devices::configure)
    })
    .bind("10.0.1.1:80")?
    .run()
    .await
}
