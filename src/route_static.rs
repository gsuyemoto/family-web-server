use actix_web::{get, post, web, Responder, HttpResponse};
use serde_json::json;
use log::{info};
use askama::Template;

#[derive(Template)] 
#[template(path = "index.html")] 

struct IndexVals<'a> {
    name: &'a str,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(index);
}

#[get("/")]
pub async fn index() -> impl Responder {

    let name = IndexVals { name: "Gary" };
    HttpResponse::Ok().body(name.render().unwrap())
}
