use crate::{models, Pool};
use crate::schema::{users};
use crate::errors::AppError;

use actix_web::{get, post, web, Responder, HttpResponse};
use diesel::prelude::*;
use askama::Template;
use log::{info, error};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(create_user)
    .service(user_profile)
    .service(get_users);
}

#[derive(Template, Queryable, Identifiable, Serialize, Debug, PartialEq)]
#[table_name="users"]
#[template(path = "profile.html")]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
}

#[get("/users/{name}")]
pub async fn user_profile (
    pool: web::Data<Pool>,
    web::Path((name)): web::Path<(String)>,
) -> HttpResponse {
    
    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    web::block(move ||
        users::table.filter(users::name.eq(name)).load::<UserProfile>(&conn))
        .await
        .map(
            |user| {
                //let profile = UserProfile { 
                //    id: user[0].id,
                //    name: user[0].name,
                //    points: user[0].points,
                //    is_admin: user[0].is_admin,
                //    devices: Vec::new(),
                //};
                HttpResponse::Ok().body(user[0].render().unwrap())
            })
        .map_err(
            |err| {
                error!("{}", err);
                HttpResponse::InternalServerError().finish()
            }).unwrap()
}

#[derive(Template)] 
#[template(path = "users.html")] 
struct GetUsers {
    names: Vec<String>,
}

#[get("/users")]
pub async fn get_users(
    pool: web::Data<Pool>,
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    web::block(move ||
        users::table.select(users::name).load::<String>(&conn))
        .await
        .map(
            |names| {
                let list = GetUsers { names };
                HttpResponse::Ok().body(list.render().unwrap())
            })
        .map_err(
            |err| {
                error!("{}", err);
                HttpResponse::InternalServerError().finish()
            }).unwrap()
}

#[post("/users")]
async fn create_user (
    form: web::Form<models::NewUser>,
    pool: web::Data<Pool>, 
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    let new_user = models::NewUser
    {
        points: 0,
        name: form.name.clone(),
        is_admin: form.is_admin,
    };

    info!("user: \n{:?}", new_user);

    web::block(move ||
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&conn))
                .await
                .map(|_| HttpResponse::Ok().finish())
                .map_err(
                    |err| {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish()
                    }).unwrap()
}
