use crate::{models, network, Pool};
use crate::schema::{users, devices};

use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;
use askama::Template;
use log::{info, error};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(create_user)
    .service(user_points)
    .service(user_profile)
    .service(get_users);
}

#[derive(Deserialize)]
struct FormPoints {
    user_id:    i32,
    old_pts:    i32,
    new_pts:    i32,
}

#[post("/user/points")]
pub async fn user_points (
    form: web::Form<FormPoints>,
    pool: web::Data<Pool>,
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

        diesel::update(users::table.filter(
            users::user_id.eq(form.user_id)))
            .set(users::points.eq(form.new_pts))
            .execute(&conn)
            .map_err(|err| {
                error!("Error updating points: {}", err);
                HttpResponse::InternalServerError().finish()
            });

        if form.old_pts == 0 && form.new_pts > 0 {
            let blocked_devices = devices::table.filter(
                devices::user_id.eq(form.user_id))
                .load::<models::Device>(&conn)
                .map_err(|err| {
                    error!("Error getting list of blocked devices: {}", err);
                    HttpResponse::InternalServerError().finish()
                }).unwrap();

            for dev in blocked_devices {
                if dev.is_blocked == 1 {
                    network::unblock_ip(&dev.addr_ip);
                }
            }
            
            diesel::update(devices::table.filter(
                    devices::user_id.eq(form.user_id)))
                    .set(devices::is_blocked.eq(0))
                    .execute(&conn)
                    .map_err(|err| {
                        error!("Error updating status of blocked devices: {}", err);
                        HttpResponse::InternalServerError().finish()
                    });
        }
    HttpResponse::Ok().finish()
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct UserProfile {
    pub user_id: i32,
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
    pub devices: Vec<(i32, String, String, i32)>,
}

#[get("/users/{name}")]
pub async fn user_profile (
    name: web::Path<(String)>,
    pool: web::Data<Pool>,
) -> HttpResponse {
    
    let name = name.into_inner();
    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    web::block(move ||
        users::table
            .filter(users::name.eq(name))
            .left_join(devices::table)
            .select((users::user_id, 
                     users::name, 
                     users::points, 
                     users::is_admin, 
                     devices::id.nullable(), 
                     devices::nickname.nullable(), 
                     devices::addr_ip.nullable(), 
                     devices::is_blocked.nullable()))
            .load::<(i32, 
                     String, 
                     i32, 
                     i32, 
                     Option<i32>, 
                     Option<String>, 
                     Option<String>, 
                     Option<i32>)>(&conn))
                .await
                .map(|query_results| {
                    if let Ok(results) = query_results {
                        let mut profile = UserProfile { 
                            user_id:    results[0].0,
                            name:       results[0].1.clone(),
                            points:     results[0].2,
                            is_admin:   results[0].3,
                            devices:    Vec::new(),
                        };

                        for r in results {
                            let device_id = r.4;

                            if let Some(device_id) = device_id {
                                profile.devices.push((device_id, 
                                                      r.5.unwrap(), 
                                                      r.6.unwrap(),
                                                      r.7.unwrap()));
                            }
                        }

                        HttpResponse::Ok().body(profile.render().unwrap())
                    }
                    else {
                        error!("Error getting user from DB.");
                        HttpResponse::InternalServerError().finish()
                    }
                })
                .map_err(|err| {
                    error!("{}", err);
                    HttpResponse::InternalServerError().finish()
                }).unwrap()
}

#[derive(Template)] 
#[template(path = "users.html")] 
struct GetUsers {
    users: Vec<(String, i32)>,
}

#[get("/users")]
pub async fn get_users(
    pool: web::Data<Pool>,
) -> HttpResponse {

    let conn = pool
                 .get()
                 .expect("couldn't get db connection from pool");

    web::block(move ||
        users::table.select((users::name, users::points)).load::<(String, i32)>(&conn))
        .await
        .map(
            |users| {
                let users = users.expect("Error getting users from db");
                let list = GetUsers { users };
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
                .map(|_| HttpResponse::SeeOther().append_header(("Location", "/users")).finish())
                .map_err(
                    |err| {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish()
                    }).unwrap()
}
