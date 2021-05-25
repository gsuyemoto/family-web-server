use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub fname: String,
    pub lname: String,
    pub is_admin: i32,
    pub num_bucks: i32,
    pub date_create: String,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct UserNew<'a> {
    pub fname: &'a str,
    pub lname: String,
    pub is_admin: i32,
    pub num_bucks: i32,
    pub date_created: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJson {
    pub fname: String,
    pub lname: String,
    pub is_admin: i32,
    pub num_bucks: i32,
    pub address: String,
}
