use crate::network;

use actix_web::{get, post, web, Responder, HttpResponse};
use serde_json::json;
use log::{info};
use askama::Template;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(index)
    .service(user_add);
}

#[derive(Template)] 
#[template(path = "index.html")] 
struct IndexVals<'a> {
    name: &'a str,
}

#[get("/")]
pub async fn index() -> impl Responder {

    let name = IndexVals { name: "Gary" };
    HttpResponse::Ok().body(name.render().unwrap())
}

#[derive(Template)] 
#[template(path = "user_add.html")] 
struct AddVals {}

#[get("/add-user")]
pub async fn user_add() -> impl Responder {

    let empty = AddVals {};
    HttpResponse::Ok().body(empty.render().unwrap())
}
