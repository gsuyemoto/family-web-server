#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::net::{Ipv4Addr};
use std::env;
use std::sync::Arc;

// use log::{debug, error, log_enabled, info, Level};
use dotenv::dotenv;

use tokio::task::{self};
use tokio::sync::Notify;

use actix_web::{middleware, App, HttpServer};
use actix_files as fs;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod route_static;
mod route_users;
mod route_devices;
mod errors;
mod models;
mod schema;
mod device_tracking;
mod network;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// static SERVER_IP: &'static str = "10.0.1.1:80";
static SERVER_IP: &'static str = "0.0.0.0:8090"; // for testing local dev

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    sudo::escalate_if_needed().expect("Unable to escalate to sudo");
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

    let notify      = Arc::new(Notify::new());
    let notify_rcv  = notify.clone();
    let db          = pool.clone();

    task::spawn(async move { device_tracking::begin_tracking(db, notify_rcv).await });

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(notify.clone())
            .wrap(middleware::Logger::default())
            .service(fs::Files::new("/js", "./templates/js"))
            .service(fs::Files::new("/css", "./templates/css"))
            .service(fs::Files::new("/assets", "./templates/assets"))
            .configure(route_users::configure)
            .configure(route_static::configure)
            .configure(route_devices::configure)
    })
    .bind(SERVER_IP)?
    .run()
    .await
}

// Create a unit test for Actix Web server
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, http::{self, StatusCode, header::ContentType}};

    #[actix_web::test]
    async fn test_index_ok() {
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_http_request();
        let resp = route_static::index(req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
