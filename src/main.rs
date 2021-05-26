#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::net::{SocketAddrV4, Ipv4Addr};

use std::env;
use dotenv::dotenv;
use actix_web::{middleware, App, HttpServer};
use listenfd::ListenFd;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod users;
mod errors;
mod models;
mod schema;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_web=info");
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

    let socket = SocketAddrV4::new(ip, port);

    // ListenFD to recompile with running server
    // systemfd --no-pid -s http::5000 -- cargo watch -x run
    let mut listenfd = ListenFd::from_env();

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .configure(users::configure)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind((socket.ip().clone(), socket.port()))?,
    };

    server
        .run()
        .await?;

    Ok(())
}
