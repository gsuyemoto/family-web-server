use crate::{models, Pool};

use actix_web::{get, post, web, Responder, HttpResponse};
use diesel::prelude::*;
use serde_derive::Deserialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(create_user);
}

#[derive(Deserialize)]
struct UserNew {
    username: String,
    is_admin: String,
}

#[post("/users")]
async fn create_user (
    form: web::Form<UserNew>,
    pool: web::Data<Pool>, 
) -> impl Responder {

    let username    = form.username.clone();
    let is_admin    = if form.is_admin == "true" {1} else {0};
    let conn        = pool
                        .get()
                        .expect("couldn't get db connection from pool");

    web::block(move || models::create_user(&conn, username, is_admin))
        .await
        .map_or_else(
            |err| {
                eprintln!("{}", err);
                HttpResponse::InternalServerError().finish()
            },
            |ok| {
                HttpResponse::Ok().json(ok)
            })
}
