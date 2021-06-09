use crate::{models, Pool};
use crate::schema::{users, devices};
use crate::errors::AppError;
use crate::network;

use actix_web::{get, post, web, http, Responder, HttpRequest, HttpResponse};
use diesel::prelude::*;
use askama::Template;
use log::{info, error, debug};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(block_device)
    .service(add_device);
}

#[derive(Deserialize)]
struct FormBlockDevice {
    name: String,
    ip: String,
}

#[post("/device/{action}")]
async fn block_device (
    web::Path((action)): web::Path<(String)>,
    form: web::Form<FormBlockDevice>,
    pool: web::Data<Pool>, 
) -> HttpResponse {

    debug!("Received action request: {}", action);
    match action.as_ref() {
        "block"     => network::block_ip(form.ip.clone()),
        "unblock"   => network::unblock_ip(form.ip.clone()),
        "remove"    => debug!("Remove request for device: {}", form.name),
        _           => debug!("Unknown request for device: {}", form.name),
    }

    HttpResponse::SeeOther()
        .header(http::header::LOCATION, format!("/users/{}", form.name))
        .finish()
}

#[derive(Deserialize)]
struct FormNewDevice {
    user_id: i32,
    nickname: String,
    is_admin: i32,
}

#[post("/users/{name}")]
async fn add_device (
    web::Path((name)): web::Path<(String)>,
    form: web::Form<FormNewDevice>,
    pool: web::Data<Pool>, 
    request: HttpRequest,
) -> HttpResponse {

    let conn        = pool
                        .get()
                        .expect("couldn't get db connection from pool");

    let ip          = request
                        .peer_addr()
                        .unwrap()
                        .ip()
                        .to_string();

    debug!("ip: {}", ip);

    let mac         = network::get_addr(Some(&ip), None).expect("Unable to find matching MAC to IP");

    debug!("mac: {}", mac);

    let new_device  = models::NewDevice
    {
        nickname: form.nickname.clone(),
        user_id: form.user_id,
        addr_mac: mac.clone(),
        addr_ip: ip,
        is_watching: 0,
        is_blocked: 0,
        is_tracked: form.is_admin,
    };

    debug!("device: \n{:?}", new_device);

    web::block(move ||
        diesel::insert_into(devices::table)
            .values(&new_device)
            .execute(&conn))
                .await
                .map(|_|
                     HttpResponse::SeeOther()
                     .header(http::header::LOCATION, format!("/users/{}", name))
                     .finish())
                .map_err(
                    |err| {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish()
                    }).unwrap()
}
