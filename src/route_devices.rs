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
    .service(add_device);
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

    let mac         = network::get_mac_from_ip(&ip).unwrap();

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
