use crate::{models, Pool};
use crate::schema::{devices};
use crate::network;

use actix_web::{get, post, web, http, HttpRequest, HttpResponse};
use diesel::prelude::*;
use askama::Template;
use log::{error, debug};

use tokio::sync::Notify;
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(remove_device)
    .service(show_devices)
    .service(block_device)
    .service(add_device);
}

#[derive(Template)] 
#[template(path = "devices.html")] 
struct GetDevices {
    devices: Vec<models::Device>,
}

#[get("/devices")]
async fn show_devices (
    pool: web::Data<Pool>, 
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    web::block(move ||
        devices::table.load::<models::Device>(&conn))
        .await
        .map(|devices| {
            if let Ok(devices) = devices {
                let list = GetDevices { devices };
                HttpResponse::Ok().body(list.render().unwrap())
            }
            else {
                error!("Error getting list of devices from DB.");
                HttpResponse::InternalServerError().finish()
            }
        })
        .map_err(|err| {
            error!("{}", err);
            HttpResponse::InternalServerError().finish()
        }).unwrap()
}

#[derive(Deserialize)]
struct FormRemoveDevice {
    name: String,
    id: String,
}

#[post("/device/remove")]
async fn remove_device (
    form: web::Form<FormRemoveDevice>,
    pool: web::Data<Pool>, 
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    let id = form.id.parse::<i32>().unwrap();
    debug!("remove device: {}", form.id);

    web::block(move ||
        diesel::delete(devices::table.filter(devices::id.eq(id)))
            .execute(&conn))
                .await
                .map(|_|
                     HttpResponse::SeeOther()
                     .append_header(("Location", format!("/users/{}", form.name)))
                     .finish())
                .map_err(
                    |err| {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish()
                    }).unwrap()
}

#[derive(Deserialize)]
struct FormBlockDevice {
    name: String,
    ip: String,
}

#[post("/device/{action}")]
async fn block_device (
    action: web::Path<String>,
    form: web::Form<FormBlockDevice>,
    pool: web::Data<Pool>, 
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    debug!("Received action request: {}", action);
    match action.into_inner().as_ref() {
        "block"     => {
            network::block_ip(&form.ip);
            diesel::update(devices::table.filter(
                    devices::addr_ip.eq(&form.ip)))
                    .set(devices::is_blocked.eq(1))
                    .execute(&conn);
        },
        "unblock"   => {
            network::unblock_ip(&form.ip);
            diesel::update(devices::table.filter(
                    devices::addr_ip.eq(&form.ip)))
                    .set(devices::is_blocked.eq(0))
                    .execute(&conn);
        },
        _           => debug!("Unknown request for device: {}", form.name),
    }

    HttpResponse::SeeOther()
        .append_header(("Location", format!("/users/{}", form.name)))
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
    name: web::Path<String>,
    form: web::Form<FormNewDevice>,
    pool: web::Data<Pool>, 
    notify: web::Data<Arc<Notify>>, 
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
        is_tracked: form.is_admin ^ 1, // XOR bitwise op
        manufacturer_name: None,
        last_checked: 0,
        last_last_checked: 0,
    };

    debug!("device: \n{:?}", new_device);
    notify.notify_one(); // let device tracking thread know that a device was added

    web::block(move ||
        diesel::insert_into(devices::table)
            .values(&new_device)
            .execute(&conn))
                .await
                .map(|_|
                     HttpResponse::SeeOther()
                     .append_header(("Location", format!("/users/{}", name)))
                     .finish())
                .map_err(
                    |err| {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish()
                    }).unwrap()
}
