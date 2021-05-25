extern crate log;
extern crate pnet;
extern crate pnet_datalink;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate actix_web;

#[macro_use]
extern crate dotenv_codegen;

#[macro_use]
extern crate diesel;

use log::{ info, debug};
use dotenv::dotenv;
use listenfd::ListenFd;

use handlebars::Handlebars;
use actix_web::{ web, App, HttpServer };
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

mod network;
mod errors;
mod models;
mod schema;
mod routes;
mod routes_db;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // ListenFD to recompile with running server
    // systemfd --no-pid -s http::5000 -- cargo watch -x run
    let mut listenfd = ListenFd::from_env();
    
    // DATABASE using SQLITE, Diesel and R2D2
    let database_url = dotenv!("DATABASE_URL");
    let database_pool = Pool::builder()
        .build(ConnectionManager::new(database_url))
        .expect("Failed to create DB pool");

    // HANDLEBARS template HTML templating system
    let mut handlebars = Handlebars::new()
        .register_templates_directory(".html", "./static/templates")
        .expect("Failed to register handlebars");
    let handlebars_ref = web::Data::new(handlebars);

    // ACTIX WEB SERVER
    let mut server = HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .wrap(errors::error_handlers())
            .app_data(handlebars_ref.clone())
            .service(routes::index)
            .service(routes::user)
            .service(routes::getid)
    });
    
    // ListenFD takes over as TCP listner
    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = dotenv!("HOST");
            let port = dotenv!("PORT");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server");
    server
        .run()
        .await?;

    Ok(())
}
