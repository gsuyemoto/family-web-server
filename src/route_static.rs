use actix_web::{get, post, web, Responder, HttpResponse};
use serde_json::json;
use handlebars::Handlebars;
use log::{info};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(index);
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>
) -> impl Responder {

    let data = json!
        ({
            "name": "Gary"
        });

    let body = hb
        .render("index", &data)
        .unwrap();

    info!("Responding with: {:?}", body);  
    HttpResponse::Ok().body(body)
}
