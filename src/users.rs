use crate::{models, Pool};
use actix_web::{get, post, web, Responder, HttpResponse};
use diesel::prelude::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(create_user)
    .service(find_user)
    .service(get_user);
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInput {
    username: String,
}

#[post("/users")]
async fn create_user (
    item: web::Json<UserInput>,
    pool: web::Data<Pool>, 
) -> impl Responder {

    let username    = item
                        .into_inner()
                        .username;
    let conn        = pool
                        .get()
                        .expect("couldn't get db connection from pool");

    web::block(move || models::create_user(&conn, &username))
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

#[get("/users/find/{name}")]
async fn find_user (
    name: web::Path<String>,
    pool: web::Data<Pool>, 
) -> impl Responder {

    let name    = name
                    .into_inner();
    let conn    = pool
                    .get()
                    .expect("couldn't get db connection from pool");

    web::block(move || {
        let key = models::UserKey::Username(name.as_str());
        models::find_user(&conn, key)
    })
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

#[get("/users/{id}")]
async fn get_user(
    user_id: web::Path<i32>,
    pool: web::Data<Pool>,
) -> impl Responder {

    let id      = user_id
                    .into_inner();
    let conn    = pool
                    .get()
                    .expect("Unable to get DB pool connection");

    web::block(move || {
        let key = models::UserKey::ID(id);
        models::find_user(&conn, key)
    })
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
